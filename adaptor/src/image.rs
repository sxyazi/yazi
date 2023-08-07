use std::{env, path::{Path, PathBuf}, sync::Arc};

use anyhow::Result;
use config::PREVIEW;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use md5::{Digest, Md5};
use shared::tty_ratio;
use tokio::fs;

pub struct Image;

impl Image {
	pub(super) async fn crop(path: &Path, size: (u16, u16)) -> Result<DynamicImage> {
		let (w, h) = {
			let r = tty_ratio();
			let (w, h) = ((size.0 as f64 * r.0) as u32, (size.1 as f64 * r.1) as u32);
			(w.min(PREVIEW.max_width), h.min(PREVIEW.max_height))
		};

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

			img.resize(w, h, FilterType::Triangle).save_with_format(cache, ImageFormat::Jpeg)?;
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

	#[inline]
	pub fn cache(path: &Path) -> PathBuf {
		env::temp_dir()
			.join("yazi")
			.join(format!("{:x}", Md5::new_with_prefix(path.to_string_lossy().as_bytes()).finalize()))
	}
}
