use std::{fs::File, io::BufReader, path::{Path, PathBuf}};

use anyhow::Result;
use exif::{In, Tag};
use image::{codecs::jpeg::JpegEncoder, imageops::{self, FilterType}, DynamicImage, Limits};
use ratatui::layout::Rect;
use yazi_config::{PREVIEW, TASKS};

use crate::Dimension;

pub struct Image;

impl Image {
	pub async fn precache(path: &Path, cache: PathBuf) -> Result<()> {
		let orientation = Self::orientation(path).await?;

		let path = path.to_owned();
		let mut img = tokio::task::spawn_blocking(move || {
			Self::set_limits(image::ImageReader::open(path)?.with_guessed_format()?).decode()
		})
		.await??;

		let (mut w, mut h) = (PREVIEW.max_width, PREVIEW.max_height);
		if (5..=8).contains(&orientation) {
			(w, h) = (h, w);
		}

		let buf = tokio::task::spawn_blocking(move || {
			if img.width() > w || img.height() > h {
				img = img.resize(w, h, Self::filter());
			}

			img = Self::rotate(img, orientation);
			if !matches!(img, DynamicImage::ImageRgb8(_)) {
				img = DynamicImage::ImageRgb8(img.into_rgb8());
			}

			let mut buf = Vec::new();
			JpegEncoder::new_with_quality(&mut buf, PREVIEW.image_quality).encode(
				img.as_bytes(),
				img.width(),
				img.height(),
				img.color().into(),
			)?;
			Ok::<_, anyhow::Error>(buf)
		})
		.await??;

		Ok(tokio::fs::write(cache, buf).await?)
	}

	pub(super) async fn downscale(path: &Path, rect: Rect) -> Result<DynamicImage> {
		let orientation = Self::orientation(path).await?;

		let path = path.to_owned();
		let mut img = tokio::task::spawn_blocking(move || {
			Self::set_limits(image::ImageReader::open(path)?.with_guessed_format()?).decode()
		})
		.await??;

		let (mut w, mut h) = Self::max_pixel(rect);
		if (5..=8).contains(&orientation) {
			(w, h) = (h, w);
		}

		// Fast path.
		if img.width() <= w && img.height() <= h && orientation <= 1 {
			return Ok(img);
		}

		tokio::task::spawn_blocking(move || {
			if img.width() > w || img.height() > h {
				img = img.resize(w, h, Self::filter())
			}
			Ok(Self::rotate(img, orientation))
		})
		.await?
	}

	pub(super) fn max_pixel(rect: Rect) -> (u32, u32) {
		Dimension::ratio()
			.map(|(r1, r2)| {
				let (w, h) = ((rect.width as f64 * r1) as u32, (rect.height as f64 * r2) as u32);
				(w.min(PREVIEW.max_width), h.min(PREVIEW.max_height))
			})
			.unwrap_or((PREVIEW.max_width, PREVIEW.max_height))
	}

	pub(super) fn pixel_area(size: (u32, u32), rect: Rect) -> Rect {
		Dimension::ratio()
			.map(|(r1, r2)| Rect {
				x:      rect.x,
				y:      rect.y,
				width:  (size.0 as f64 / r1).ceil() as u16,
				height: (size.1 as f64 / r2).ceil() as u16,
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

	async fn orientation(path: &Path) -> Result<u8> {
		// We don't want to read the orientation of the cached image that has been
		// rotated in the `Self::precache()` step.
		if path.parent() == Some(&PREVIEW.cache_dir) {
			return Ok(0);
		}

		let path = path.to_owned();
		tokio::task::spawn_blocking(move || {
			let file = std::fs::File::open(path)?;

			let mut reader = std::io::BufReader::new(&file);
			let Ok(exif) = exif::Reader::new().read_from_container(&mut reader) else {
				return Ok(0);
			};

			Ok(match exif.get_field(Tag::Orientation, In::PRIMARY) {
				Some(orientation) => match orientation.value.get_uint(0) {
					Some(v @ 1..=8) => v as u8,
					_ => 1,
				},
				None => 1,
			})
		})
		.await?
	}

	// https://magnushoff.com/articles/jpeg-orientation/
	fn rotate(mut img: DynamicImage, orientation: u8) -> DynamicImage {
		let rgba = img.color().has_alpha();
		img = match orientation {
			2 => DynamicImage::ImageRgba8(imageops::flip_horizontal(&img)),
			3 => DynamicImage::ImageRgba8(imageops::rotate180(&img)),
			4 => DynamicImage::ImageRgba8(imageops::flip_vertical(&img)),
			5 => DynamicImage::ImageRgba8(imageops::flip_horizontal(&imageops::rotate90(&img))),
			6 => DynamicImage::ImageRgba8(imageops::rotate90(&img)),
			7 => DynamicImage::ImageRgba8(imageops::flip_horizontal(&imageops::rotate270(&img))),
			8 => DynamicImage::ImageRgba8(imageops::rotate270(&img)),
			_ => img,
		};
		if !rgba {
			img = DynamicImage::ImageRgb8(img.into_rgb8());
		}
		img
	}

	fn set_limits(mut r: image::ImageReader<BufReader<File>>) -> image::ImageReader<BufReader<File>> {
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
		r.limits(limits);
		r
	}
}
