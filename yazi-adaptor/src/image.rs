use std::{path::Path, sync::Arc};

use anyhow::Result;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use tokio::fs;
use yazi_config::PREVIEW;
use yazi_shared::Term;

pub struct Image;

impl Image {
	pub(super) async fn crop(path: &Path, size: (u16, u16)) -> Result<DynamicImage> {
		let (w, h) = Term::ratio()
			.map(|(w, h)| {
				let (w, h) = ((size.0 as f64 * w) as u32, (size.1 as f64 * h) as u32);
				(w.min(PREVIEW.max_width), h.min(PREVIEW.max_height))
			})
			.unwrap_or((PREVIEW.max_width, PREVIEW.max_height));

		let img = fs::read(path).await?;
		let img = tokio::task::spawn_blocking(move || -> Result<DynamicImage> {
			let img = image::load_from_memory(&img)?;
			Ok(if img.width() > w || img.height() > h {
				img.resize(w, h, FilterType::Triangle)
			} else {
				img
			})
		});

		img.await?
	}

	pub async fn precache(img: Arc<Vec<u8>>, cache: impl AsRef<Path>) -> Result<bool> {
		let cache = cache.as_ref().to_owned();
		let result = tokio::task::spawn_blocking(move || {
			let img = image::load_from_memory(&img)?;
			let (w, h) = (PREVIEW.max_width, PREVIEW.max_height);

			if img.width() <= w && img.height() <= h {
				return Ok(false);
			}

			match img.resize(w, h, FilterType::Triangle) {
				DynamicImage::ImageRgb8(buf) => buf.save_with_format(cache, ImageFormat::Jpeg),
				DynamicImage::ImageRgba8(buf) => buf.save_with_format(cache, ImageFormat::Jpeg),
				buf => buf.to_rgb8().save_with_format(cache, ImageFormat::Jpeg),
			}?;

			Ok(true)
		});

		result.await?
	}

	#[inline]
	pub async fn precache_anyway(img: Arc<Vec<u8>>, cache: impl AsRef<Path>) -> Result<()> {
		Ok(match Self::precache(img.clone(), &cache).await {
			Ok(true) => (),
			_ => fs::write(cache, &*img).await?,
		})
	}
}
