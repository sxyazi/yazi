use std::{io::Write, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::{imageops::FilterType, DynamicImage};
use ratatui::prelude::Rect;
use tokio::{fs, io::AsyncWriteExt};

use crate::{config::PREVIEW, misc::tty_ratio, ui::Term};

pub struct Kitty;

impl Kitty {
	pub async fn image_show(path: &Path, rect: Rect) -> Result<()> {
		let (w, h) = {
			let r = tty_ratio();
			let (w, h) = ((rect.width as f64 * r.0) as u32, (rect.height as f64 * r.1) as u32);
			(w.min(PREVIEW.max_width), h.min(PREVIEW.max_height))
		};

		let img = fs::read(path).await?;
		let b = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
			let img = image::load_from_memory(&img)?;
			Self::encode(if img.width() > w || img.height() > h {
				img.resize(w, h, FilterType::Triangle)
			} else {
				img
			})
		})
		.await??;

		Term::move_to(rect.x, rect.y).ok();
		tokio::io::stdout().write_all(&b).await.ok();
		Ok(())
	}

	#[inline]
	pub fn image_hide() { std::io::stdout().write_all(b"\x1b\\\x1b_Ga=d\x1b\\").ok(); }

	fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		fn output(raw: Vec<u8>, format: u8, size: (u32, u32)) -> Result<Vec<u8>> {
			let b64 = general_purpose::STANDARD.encode(raw).chars().collect::<Vec<_>>();

			let mut it = b64.chunks(4096).peekable();
			let mut buf = Vec::with_capacity(b64.len() + it.len() * 50);
			if let Some(first) = it.next() {
				write!(
					buf,
					"\x1b\\\x1b_Ga=d\x1b\\\x1b_Ga=T,f={},s={},v={},m={};{}\x1b\\",
					format,
					size.0,
					size.1,
					it.peek().is_some() as u8,
					first.iter().collect::<String>(),
				)?;
			}

			while let Some(chunk) = it.next() {
				write!(
					buf,
					"\x1b_Gm={};{}\x1b\\",
					it.peek().is_some() as u8,
					chunk.iter().collect::<String>()
				)?;
			}
			Ok(buf)
		}

		let size = (img.width(), img.height());
		match img {
			DynamicImage::ImageRgb8(v) => output(v.into_raw(), 24, size),
			DynamicImage::ImageRgba8(v) => output(v.into_raw(), 32, size),
			v => output(v.to_rgb8().into_raw(), 24, size),
		}
	}
}
