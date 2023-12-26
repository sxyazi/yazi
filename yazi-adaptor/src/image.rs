use std::{fs::File, io::BufReader, path::{Path, PathBuf}};

use anyhow::Result;
use image::{imageops::FilterType, io::Limits, DynamicImage, ImageFormat};
use ratatui::layout::Rect;
use yazi_config::{PREVIEW, TASKS};
use yazi_shared::term::Term;

pub struct Image;

impl Image {
	pub async fn precache(path: &Path, cache: PathBuf) -> Result<()> {
		let path = path.to_owned();
		let mut img = tokio::task::spawn_blocking(move || {
			Self::set_limits(image::io::Reader::open(path)?.with_guessed_format()?).decode()
		})
		.await??;

		tokio::task::spawn_blocking(move || {
			let (w, h) = (PREVIEW.max_width, PREVIEW.max_height);
			if img.width() > w || img.height() > h {
				img = img.resize(w, h, FilterType::Triangle);
			}

			Ok(match img {
				DynamicImage::ImageRgb8(buf) => buf.save_with_format(cache, ImageFormat::Jpeg),
				DynamicImage::ImageRgba8(buf) => buf.save_with_format(cache, ImageFormat::Jpeg),
				buf => buf.into_rgb8().save_with_format(cache, ImageFormat::Jpeg),
			}?)
		})
		.await?
	}

	pub(super) async fn downscale(path: &Path, rect: Rect) -> Result<DynamicImage> {
		let path = path.to_owned();
		let img = tokio::task::spawn_blocking(move || {
			Self::set_limits(image::io::Reader::open(path)?.with_guessed_format()?).decode()
		})
		.await??;

		let (w, h) = Self::max_size(rect);
		tokio::task::spawn_blocking(move || {
			Ok(if img.width() > w || img.height() > h {
				img.resize(w, h, FilterType::Triangle)
			} else {
				img
			})
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
