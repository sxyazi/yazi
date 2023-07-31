use std::path::Path;

use anyhow::Result;
use config::PREVIEW;
use image::{imageops::FilterType, DynamicImage};
use shared::tty_ratio;
use tokio::fs;

pub(super) struct Image;

impl Image {
	pub(super) async fn resize(path: &Path, size: (u16, u16)) -> Result<DynamicImage> {
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

		Ok(img.await??)
	}
}
