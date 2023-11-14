use std::{io::{stdout, Write}, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::DynamicImage;
use ratatui::prelude::Rect;
use yazi_shared::Term;

use super::image::Image;
use crate::{CLOSE, ESCAPE, START};

pub(super) struct KittyOld;

impl KittyOld {
	pub(super) async fn image_show(path: &Path, rect: Rect) -> Result<()> {
		let img = Image::crop(path, (rect.width, rect.height)).await?;
		let b = Self::encode(img).await?;

		Self::image_hide()?;
		Term::move_lock(stdout().lock(), (rect.x, rect.y), |stdout| Ok(stdout.write_all(&b)?))
	}

	#[inline]
	pub(super) fn image_hide() -> Result<()> {
		let mut stdout = stdout().lock();
		stdout.write_all(format!("{}_Gq=1,a=d,d=A{}\\{}", START, ESCAPE, CLOSE).as_bytes())?;
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
					"{}_Gq=1,a=T,z=-1,C=1,f={},s={},v={},m={};{}{}\\{}",
					START,
					format,
					size.0,
					size.1,
					it.peek().is_some() as u8,
					first.iter().collect::<String>(),
					ESCAPE,
					CLOSE
				)?;
			}

			while let Some(chunk) = it.next() {
				write!(
					buf,
					"{}_Gm={};{}{}\\{}",
					START,
					it.peek().is_some() as u8,
					chunk.iter().collect::<String>(),
					ESCAPE,
					CLOSE
				)?;
			}

			buf.write_all(CLOSE.as_bytes())?;
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
