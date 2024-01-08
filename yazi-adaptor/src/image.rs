use std::{fs::File, io::BufReader, ops::RangeBounds, path::{Path, PathBuf}};

use anyhow::Result;
use exif::{In, Tag};
use image::{imageops::{self, FilterType}, io::Limits, DynamicImage, ImageFormat};
use ratatui::layout::Rect;
use yazi_config::{PREVIEW, TASKS};
use yazi_shared::term::Term;

pub struct Image;

impl Image {
	pub async fn precache(path: &Path, cache: PathBuf) -> Result<()> {
		let orientation = Self::orientation(path).await?;

		let path = path.to_owned();
		let mut img = tokio::task::spawn_blocking(move || {
			Self::set_limits(image::io::Reader::open(path)?.with_guessed_format()?).decode()
		})
		.await??;

		let (mut w, mut h) = (PREVIEW.max_width, PREVIEW.max_height);
		tokio::task::spawn_blocking(move || {
			if (5..=8).contains(&orientation) {
				(w, h) = (h, w);
			}

			if img.width() > w || img.height() > h {
				img = img.resize(w, h, FilterType::Triangle);
			}

			img = Self::rotate(img, orientation);
			Ok(match img {
				DynamicImage::ImageRgb8(buf) => buf.save_with_format(cache, ImageFormat::Jpeg),
				DynamicImage::ImageRgba8(buf) => buf.save_with_format(cache, ImageFormat::Jpeg),
				buf => buf.into_rgb8().save_with_format(cache, ImageFormat::Jpeg),
			}?)
		})
		.await?
	}

	pub(super) async fn downscale(path: &Path, rect: Rect) -> Result<DynamicImage> {
		let orientation = Self::orientation(path).await?;

		let path = path.to_owned();
		let mut img = tokio::task::spawn_blocking(move || {
			Self::set_limits(image::io::Reader::open(path)?.with_guessed_format()?).decode()
		})
		.await??;

		let (mut w, mut h) = Self::max_size(rect);
		tokio::task::spawn_blocking(move || {
			if (5..=8).contains(&orientation) {
				(w, h) = (h, w);
			}

			if img.width() > w || img.height() > h {
				img = img.resize(w, h, FilterType::Triangle)
			}

			Ok(Self::rotate(img, orientation))
		})
		.await?
	}

	pub(super) fn max_size(rect: Rect) -> (u32, u32) {
		Term::ratio()
			.map(|(r1, r2)| {
				let (w, h) = ((rect.width as f64 * r1) as u32, (rect.height as f64 * r2) as u32);
				(w.min(PREVIEW.max_width), h.min(PREVIEW.max_height))
			})
			.unwrap_or((PREVIEW.max_width, PREVIEW.max_height))
	}

	async fn orientation(path: &Path) -> Result<u8> {
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
		if orientation == 2 {
			img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
		} else if orientation == 3 {
			img = DynamicImage::ImageRgba8(imageops::rotate180(&img));
		} else if orientation == 4 {
			img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
		} else if orientation == 5 {
			img = DynamicImage::ImageRgba8(imageops::rotate90(&img));
			img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
		} else if orientation == 6 {
			img = DynamicImage::ImageRgba8(imageops::rotate90(&img));
		} else if orientation == 7 {
			img = DynamicImage::ImageRgba8(imageops::rotate270(&img));
			img = DynamicImage::ImageRgba8(imageops::flip_horizontal(&img));
		} else if orientation == 8 {
			img = DynamicImage::ImageRgba8(imageops::rotate270(&img));
		}
		if !rgba {
			img = DynamicImage::ImageRgb8(img.into_rgb8());
		}
		img
	}

	fn set_limits(mut r: image::io::Reader<BufReader<File>>) -> image::io::Reader<BufReader<File>> {
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
