use std::path::{Path, PathBuf};

use anyhow::Result;
use fast_image_resize::images::Image as FirImage;
use fast_image_resize::{IntoImageView, ResizeAlg, ResizeOptions, Resizer, create_srgb_mapper};
use image::{DynamicImage, ImageDecoder, ImageError, ImageReader, Limits, codecs::{jpeg::JpegEncoder, png::PngEncoder}, metadata::Orientation};
use ratatui::layout::Rect;
use yazi_config::YAZI;
use yazi_emulator::Dimension;
use yazi_fs::provider::{Provider, local::Local};

use crate::Icc;

pub struct Image;

impl Image {
	pub async fn precache(src: PathBuf, cache: &Path) -> Result<()> {
		let (mut img, orientation) = Self::decode_from(src).await?;
		let (w, h) = Self::flip_size(orientation, (YAZI.preview.max_width, YAZI.preview.max_height));

		let buf = tokio::task::spawn_blocking(move || {
			if img.width() > w || img.height() > h {
				img = Self::fir_resize(img, w, h, Self::resize_alg())?;
			}
			if orientation != Orientation::NoTransforms {
				img.apply_orientation(orientation);
			}

			let mut buf = Vec::new();
			if img.color().has_alpha() {
				let encoder = PngEncoder::new(&mut buf);
				img.write_with_encoder(encoder)?;
			} else {
				let encoder = JpegEncoder::new_with_quality(&mut buf, YAZI.preview.image_quality);
				img.write_with_encoder(encoder)?;
			}

			Ok::<_, anyhow::Error>(buf)
		})
		.await??;

		Ok(Local::regular(&cache).write(buf).await?)
	}

	pub(super) async fn downscale(path: PathBuf, rect: Rect) -> Result<DynamicImage> {
		let (mut img, orientation) = Self::decode_from(path).await?;
		let (w, h) = Self::flip_size(orientation, Self::max_pixel(rect));

		// Fast path.
		if img.width() <= w && img.height() <= h && orientation == Orientation::NoTransforms {
			return Ok(img);
		}

		let img = tokio::task::spawn_blocking(move || {
			if img.width() > w || img.height() > h {
				img = Self::fir_resize(img, w, h, Self::resize_alg())?;
			}
			if orientation != Orientation::NoTransforms {
				img.apply_orientation(orientation);
			}
			Ok::<_, anyhow::Error>(img)
		})
		.await??;

		Ok(img)
	}

	pub(super) fn max_pixel(rect: Rect) -> (u16, u16) {
		Dimension::cell_size()
			.map(|(cw, ch)| {
				let (w, h) = ((rect.width as f64 * cw) as u16, (rect.height as f64 * ch) as u16);
				(w.min(YAZI.preview.max_width), h.min(YAZI.preview.max_height))
			})
			.unwrap_or((YAZI.preview.max_width, YAZI.preview.max_height))
	}

	pub(super) fn pixel_area(size: (u32, u32), rect: Rect) -> Rect {
		Dimension::cell_size()
			.map(|(cw, ch)| Rect {
				x:      rect.x,
				y:      rect.y,
				width:  (size.0 as f64 / cw).ceil() as u16,
				height: (size.1 as f64 / ch).ceil() as u16,
			})
			.unwrap_or(rect)
	}

	fn resize_alg() -> ResizeAlg {
		use fast_image_resize::FilterType;
		match YAZI.preview.image_filter.as_str() {
			"nearest" => ResizeAlg::Nearest,
			"triangle" => ResizeAlg::Convolution(FilterType::Bilinear),
			"catmull-rom" => ResizeAlg::Convolution(FilterType::CatmullRom),
			"gaussian" => ResizeAlg::Convolution(FilterType::Gaussian),
			"lanczos3" => ResizeAlg::Convolution(FilterType::Lanczos3),
			other => {
				tracing::warn!("unknown image_filter {other:?}, falling back to lanczos3");
				ResizeAlg::Convolution(FilterType::Lanczos3)
			}
		}
	}

	fn fir_resize(img: DynamicImage, w: u32, h: u32, alg: ResizeAlg) -> anyhow::Result<DynamicImage> {
		let img = match img.pixel_type() {
			Some(_) => img,
			None => {
				tracing::debug!("converting exotic pixel type to rgb8 for resize");
				img.to_rgb8().into()
			}
		};

		let pixel_type = img
			.pixel_type()
			.ok_or_else(|| anyhow::anyhow!("unsupported pixel type for fast_image_resize"))?;

		if matches!(alg, ResizeAlg::Nearest) {
			let mut dst = FirImage::new(w, h, pixel_type);
			let mut resizer = Resizer::new();
			resizer.resize(&img, &mut dst, &ResizeOptions::new().resize_alg(alg))?;
			return Self::reconstruct(w, h, pixel_type, dst.into_vec());
		}

		let mapper = create_srgb_mapper();

		// sRGB -> linear
		let mut linear_src = FirImage::new(img.width(), img.height(), pixel_type);
		mapper.forward_map(&img, &mut linear_src)?;

		// Resize in linear light
		let mut dst = FirImage::new(w, h, pixel_type);
		let mut resizer = Resizer::new();
		let opts = ResizeOptions::new().resize_alg(alg);
		resizer.resize(&linear_src, &mut dst, &opts)?;

		// linear -> sRGB
		let mut srgb_dst = FirImage::new(w, h, pixel_type);
		mapper.backward_map(&dst, &mut srgb_dst)?;

		Self::reconstruct(w, h, pixel_type, srgb_dst.into_vec())
	}

	fn reconstruct(
		w: u32,
		h: u32,
		pixel_type: fast_image_resize::PixelType,
		buf: Vec<u8>,
	) -> anyhow::Result<DynamicImage> {
		use anyhow::Context;
		use fast_image_resize::PixelType;
		use image::{GrayAlphaImage, GrayImage, RgbImage, RgbaImage};
		match pixel_type {
			PixelType::U8 => {
				Ok(GrayImage::from_raw(w, h, buf).context("gray reconstruct")?.into())
			}
			PixelType::U8x2 => {
				Ok(GrayAlphaImage::from_raw(w, h, buf).context("graya reconstruct")?.into())
			}
			PixelType::U8x3 => {
				Ok(RgbImage::from_raw(w, h, buf).context("rgb reconstruct")?.into())
			}
			PixelType::U8x4 => {
				Ok(RgbaImage::from_raw(w, h, buf).context("rgba reconstruct")?.into())
			}
			_ => Err(anyhow::anyhow!("unexpected pixel type {pixel_type:?}")),
		}
	}

	async fn decode_from(path: PathBuf) -> Result<(DynamicImage, Orientation)> {
		let mut limits = Limits::no_limits();
		if YAZI.tasks.image_alloc > 0 {
			limits.max_alloc = Some(YAZI.tasks.image_alloc as u64);
		}
		if YAZI.tasks.image_bound[0] > 0 {
			limits.max_image_width = Some(YAZI.tasks.image_bound[0] as u32);
		}
		if YAZI.tasks.image_bound[1] > 0 {
			limits.max_image_height = Some(YAZI.tasks.image_bound[1] as u32);
		}

		tokio::task::spawn_blocking(move || {
			let mut reader = ImageReader::open(path)?;
			reader.limits(limits);

			let mut decoder = reader.with_guessed_format()?.into_decoder()?;
			let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
			Ok((Icc::transform(decoder)?, orientation))
		})
		.await
		.map_err(|e| ImageError::IoError(e.into()))?
	}

	fn flip_size(orientation: Orientation, (w, h): (u16, u16)) -> (u32, u32) {
		use image::metadata::Orientation::{Rotate90, Rotate90FlipH, Rotate270, Rotate270FlipH};
		match orientation {
			Rotate90 | Rotate270 | Rotate90FlipH | Rotate270FlipH => (h as u32, w as u32),
			_ => (w as u32, h as u32),
		}
	}
}
