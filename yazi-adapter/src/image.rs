use std::path::{Path, PathBuf};

use anyhow::Result;
use image::{DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageError, ImageReader, ImageResult, Limits, codecs::{jpeg::JpegEncoder, png::PngEncoder}, imageops::FilterType, metadata::Orientation};
use ratatui::layout::{Rect, Size};
use yazi_config::{PREVIEW, TASKS};

use crate::{Dimension, Offset};

pub struct Image;

impl Image {
	pub async fn precache(path: &Path, cache: PathBuf) -> Result<()> {
		let (mut img, orientation, icc) = Self::decode_from(path).await?;
		let (w, h) = Self::flip_size(orientation, (PREVIEW.max_width, PREVIEW.max_height));

		let buf = tokio::task::spawn_blocking(move || {
			if img.width() > w || img.height() > h {
				img = img.resize(w, h, Self::filter());
			}
			if orientation != Orientation::NoTransforms {
				img.apply_orientation(orientation);
			}

			let mut buf = Vec::new();
			if img.color().has_alpha() {
				let rgba = img.into_rgba8();
				let mut encoder = PngEncoder::new(&mut buf);
				icc.map(|b| encoder.set_icc_profile(b));
				encoder.write_image(&rgba, rgba.width(), rgba.height(), ExtendedColorType::Rgba8)?;
			} else {
				let mut encoder = JpegEncoder::new_with_quality(&mut buf, PREVIEW.image_quality);
				icc.map(|b| encoder.set_icc_profile(b));
				encoder.encode_image(&img.into_rgb8())?;
			}

			Ok::<_, ImageError>(buf)
		})
		.await??;

		Ok(tokio::fs::write(cache, buf).await?)
	}

	pub(super) async fn downscale(path: &Path, rect: Rect) -> Result<DynamicImage> {
		let (mut img, orientation, _) = Self::decode_from(path).await?;
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

	pub(super) fn max_pixel(rect: Rect) -> (u32, u32) {
		Dimension::ratio()
			.map(|(r1, r2)| {
				let (w, h) = ((rect.width as f64 * r1) as u32, (rect.height as f64 * r2) as u32);
				(w.min(PREVIEW.max_width), h.min(PREVIEW.max_height))
			})
			.unwrap_or((PREVIEW.max_width, PREVIEW.max_height))
	}

	pub(super) fn pixel_area(size: (u32, u32), rect: Rect, offset: Option<Offset>) -> Rect {
		Dimension::ratio()
			.map(|(r1, r2)| {
				let width = (size.0 as f64 / r1).ceil() as u16;
				let height = (size.1 as f64 / r2).ceil() as u16;
				let offset = offset.unwrap_or_else(|| {
					Offset::from((Size { width, height }, Size { width: rect.width, height: rect.height }))
				});
				Rect { x: rect.x + offset.x, y: rect.y + offset.y, width, height }
			})
			.unwrap_or(rect)
	}

	#[inline]
	fn filter() -> FilterType {
		match PREVIEW.image_filter.as_str() {
			"nearest" => FilterType::Nearest,
			"triangle" => FilterType::Triangle,
			"catmull-rom" => FilterType::CatmullRom,
			"gaussian" => FilterType::Gaussian,
			"lanczos3" => FilterType::Lanczos3,
			_ => FilterType::Triangle,
		}
	}

	async fn decode_from(path: &Path) -> ImageResult<(DynamicImage, Orientation, Option<Vec<u8>>)> {
		let mut limits = Limits::no_limits();
		if TASKS.image_alloc > 0 {
			limits.max_alloc = Some(TASKS.image_alloc as u64);
		}
		if TASKS.image_bound[0] > 0 {
			limits.max_image_width = Some(TASKS.image_bound[0] as u32);
		}
		if TASKS.image_bound[1] > 0 {
			limits.max_image_height = Some(TASKS.image_bound[1] as u32);
		}

		let path = path.to_owned();
		tokio::task::spawn_blocking(move || {
			let mut reader = ImageReader::open(path)?;
			reader.limits(limits);

			let mut decoder = reader.with_guessed_format()?.into_decoder()?;
			let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
			let icc = decoder.icc_profile().unwrap_or_default();

			Ok((DynamicImage::from_decoder(decoder)?, orientation, icc))
		})
		.await
		.map_err(|e| ImageError::IoError(e.into()))?
	}

	fn flip_size(orientation: Orientation, (w, h): (u32, u32)) -> (u32, u32) {
		use image::metadata::Orientation::{Rotate90, Rotate90FlipH, Rotate270, Rotate270FlipH};
		match orientation {
			Rotate90 | Rotate270 | Rotate90FlipH | Rotate270FlipH => (h, w),
			_ => (w, h),
		}
	}
}
