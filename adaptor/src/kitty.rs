use std::{io::{stdout, Write}, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::DynamicImage;
use ratatui::prelude::Rect;
use shared::Term;

use super::image::Image;

pub(super) struct Kitty;

impl Kitty {
	pub(super) async fn image_show(path: &Path, rect: Rect) -> Result<()> {
		let img = Image::crop(path, (rect.width, rect.height)).await?;
		let b = Self::encode(img).await?;

		Self::image_hide()?;
		Term::move_lock(stdout().lock(), (rect.x, rect.y), |stdout| Ok(stdout.write_all(&b)?))
	}

	#[inline]
	pub(super) fn image_hide() -> Result<()> {
		let mut stdout = stdout().lock();
		stdout.write_all(b"\x1b_Ga=d,d=A\x1b\\")?;
		stdout.flush()?;
		Ok(())
	}

	async fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		fn output(raw: &[u8], format: u8, size: (u32, u32)) -> Result<Vec<u8>> {
			let b64 = general_purpose::STANDARD.encode(raw).chars().collect::<Vec<_>>();

			let mut it = b64.chunks(4096).peekable();
			let mut buf = Vec::with_capacity(b64.len() + it.len() * 50);
			if let Some(first) = it.next() {
				write!(
					buf,
					"\x1b_Ga=T,f={},s={},v={},m={};{}\x1b\\",
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
		tokio::task::spawn_blocking(move || match img {
			DynamicImage::ImageRgb8(v) => output(v.as_raw(), 24, size),
			DynamicImage::ImageRgba8(v) => output(v.as_raw(), 32, size),
			v => output(v.to_rgb8().as_raw(), 24, size),
		})
		.await?
	}
}
