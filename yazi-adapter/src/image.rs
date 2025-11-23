use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::{
	DynamicImage, ImageBuffer, ImageDecoder, ImageError, ImageReader, Limits, RgbImage, RgbaImage,
	codecs::{jpeg::JpegEncoder, png::PngEncoder},
	imageops::FilterType,
	metadata::{Cicp, Orientation},
};
use qcms::{Profile, Transform};
use ratatui::layout::Rect;
use yazi_config::YAZI;
use yazi_fs::provider::{Provider, local::Local};

use crate::Dimension;

pub struct Image;

impl Image {
	pub async fn precache(src: PathBuf, cache: &Path) -> Result<()> {
		let (mut img, orientation) = Self::decode_from(src).await?;
		let (w, h) = Self::flip_size(orientation, (YAZI.preview.max_width, YAZI.preview.max_height));

		let buf = tokio::task::spawn_blocking(move || {
			if img.width() > w || img.height() > h {
				img = img.resize(w, h, Self::filter());
			}
			if orientation != Orientation::NoTransforms {
				img.apply_orientation(orientation);
			}

			let mut buf = Vec::new();
			if img.color().has_alpha() {
				let encoder = PngEncoder::new(&mut buf);
				img.write_with_encoder(encoder)?
			} else {
				let encoder = JpegEncoder::new_with_quality(&mut buf, YAZI.preview.image_quality);
				img.write_with_encoder(encoder)?
			}

			Ok::<_, ImageError>(buf)
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
				img = img.resize(w, h, Self::filter())
			}
			if orientation != Orientation::NoTransforms {
				img.apply_orientation(orientation);
			}
			img
		})
		.await?;

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
				x: rect.x,
				y: rect.y,
				width: (size.0 as f64 / cw).ceil() as u16,
				height: (size.1 as f64 / ch).ceil() as u16,
			})
			.unwrap_or(rect)
	}

	fn filter() -> FilterType {
		match YAZI.preview.image_filter.as_str() {
			"nearest" => FilterType::Nearest,
			"triangle" => FilterType::Triangle,
			"catmull-rom" => FilterType::CatmullRom,
			"gaussian" => FilterType::Gaussian,
			"lanczos3" => FilterType::Lanczos3,
			_ => FilterType::Triangle,
		}
	}

	fn _decode(mut decoder: impl ImageDecoder) -> Result<DynamicImage> {
		let data_type = match decoder.color_type() {
			image::ColorType::L8 => Some(qcms::DataType::Gray8),
			image::ColorType::La8 => Some(qcms::DataType::GrayA8),
			image::ColorType::Rgb8 => Some(qcms::DataType::RGB8),
			image::ColorType::Rgba8 => Some(qcms::DataType::RGBA8),
			_ => None,
		};
		let icc = decoder.icc_profile().unwrap_or_default();

		if let Some(dt) = data_type
			&& let Some(icc_profile) = icc
			&& let Some(input) = Profile::new_from_slice(&icc_profile, false)
			&& !input.is_sRGB()
		{
			let mut data = vec![0u8; decoder.total_bytes() as usize];
			let (w, h) = decoder.dimensions();
			let has_alpha = decoder.color_type().has_alpha();
			let mut output = Profile::new_sRGB();
			output.precache_output_transform();
			decoder.read_image(&mut data)?;

			let xfm = Transform::new(&input, &output, dt, qcms::Intent::default())
				.context("Couldn't make a profile transformer")?;

			xfm.apply(&mut data);

			let mut image = if has_alpha {
				let buf =
					RgbaImage::from_raw(w, h, data).context("Couldn't read the transformed image data")?;
				DynamicImage::ImageRgba8(buf)
			} else {
				let buf =
					RgbImage::from_raw(w, h, data).context("Couldn't read the transformed image data")?;
				DynamicImage::ImageRgb8(buf)
			};

			image.set_rgb_primaries(Cicp::SRGB.primaries);
			image.set_transfer_function(Cicp::SRGB.transfer);
			Ok(image)
		} else {
			Ok(DynamicImage::from_decoder(decoder)?)
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
			Ok((Self::_decode(decoder)?, orientation))
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
