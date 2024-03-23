use core::str;
use std::{io::{stderr, Write}, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::DynamicImage;
use ratatui::layout::Rect;
use yazi_shared::term::Term;

use super::image::Image;
use crate::{adaptor::Adaptor, CLOSE, ESCAPE, START};

pub(super) struct KittyOld;

impl KittyOld {
	pub(super) async fn image_show(path: &Path, rect: Rect) -> Result<(u32, u32)> {
		let img = Image::downscale(path, rect).await?;
		let size = (img.width(), img.height());
		let b = Self::encode(img).await?;

		Adaptor::KittyOld.image_hide()?;
		Adaptor::shown_store(rect, size);
		Term::move_lock(stderr().lock(), (rect.x, rect.y), |stderr| {
			stderr.write_all(&b)?;
			Ok(size)
		})
	}

	#[inline]
	pub(super) fn image_erase() -> Result<()> {
		let mut stderr = stderr().lock();
		write!(stderr, "{}_Gq=1,a=d,d=A{}\\{}", START, ESCAPE, CLOSE)?;
		stderr.flush()?;
		Ok(())
	}

	async fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		fn output(raw: &[u8], format: u8, size: (u32, u32)) -> Result<Vec<u8>> {
			let b64 = general_purpose::STANDARD.encode(raw).into_bytes();

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
					unsafe { str::from_utf8_unchecked(first) },
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
					unsafe { str::from_utf8_unchecked(chunk) },
					ESCAPE,
					CLOSE
				)?;
			}

			write!(buf, "{}", CLOSE)?;
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
