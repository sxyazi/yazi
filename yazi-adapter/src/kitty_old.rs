use core::str;
use std::{io::{stderr, LineWriter, Write}, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::DynamicImage;
use ratatui::layout::Rect;

use super::image::Image;
use crate::{adapter::Adapter, Emulator, CLOSE, ESCAPE, START};

pub(super) struct KittyOld;

impl KittyOld {
	pub(super) async fn image_show(path: &Path, max: Rect) -> Result<Rect> {
		let img = Image::downscale(path, max).await?;
		let area = Image::pixel_area((img.width(), img.height()), max);
		let b = Self::encode(img).await?;

		Adapter::KittyOld.image_hide()?;
		Adapter::shown_store(area);
		Emulator::move_lock((area.x, area.y), |stderr| {
			stderr.write_all(&b)?;
			Ok(area)
		})
	}

	#[inline]
	pub(super) fn image_erase(_: Rect) -> Result<()> {
		let mut stderr = LineWriter::new(stderr());
		write!(stderr, "{}_Gq=2,a=d,d=A{}\\{}", START, ESCAPE, CLOSE)?;
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
					"{}_Gq=2,a=T,z=-1,C=1,f={},s={},v={},m={};{}{}\\{}",
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
			v => output(v.into_rgb8().as_raw(), 24, size),
		})
		.await?
	}
}
