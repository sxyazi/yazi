use std::{path::Path, sync::Arc};

use anyhow::Result;
use config::PREVIEW;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use shared::Term;
use tokio::fs;

pub struct Image;

impl Image {
	pub(super) async fn crop(
		path: &Path,
		(target_width, target_height): (u16, u16),
	) -> Result<DynamicImage> {
		let (max_width, max_height) = Term::ratio()
			.map(|(a, b)| {
				let (width, height) = ((target_width as f64 * a) as u32, (target_height as f64 * b) as u32);
				(width.min(PREVIEW.max_width), height.min(PREVIEW.max_height))
			})
			.unwrap_or((PREVIEW.max_width, PREVIEW.max_height));

		let img = fs::read(path).await?;
		let img = tokio::task::spawn_blocking(move || -> Result<DynamicImage> {
			let img = image::load_from_memory(&img)?;
			Ok(if img.width() > max_width || img.height() > max_height {
				img.resize(max_width, max_height, FilterType::Triangle)
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
			let (max_width, max_height) = (PREVIEW.max_width, PREVIEW.max_height);

			if img.width() <= max_width && img.height() <= max_height {
				return Ok(false);
			}

			img
				.resize(max_width, max_height, FilterType::Triangle)
				.save_with_format(cache, ImageFormat::Jpeg)?;
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
