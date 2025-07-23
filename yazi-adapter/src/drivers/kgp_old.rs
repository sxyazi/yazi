use core::str;
use std::io::Write;

use anyhow::Result;
use base64::{Engine, engine::general_purpose};
use image::DynamicImage;
use ratatui::layout::Rect;
use yazi_shared::url::Url;
use yazi_term::tty::TTY;

use crate::{CLOSE, ESCAPE, Emulator, Image, START, adapter::Adapter};

pub(crate) struct KgpOld;

impl KgpOld {
	pub(crate) async fn image_show(url: &Url, max: Rect) -> Result<Rect> {
		let img = Image::downscale(url, max).await?;
		let area = Image::pixel_area((img.width(), img.height()), max);
		let b = Self::encode(img).await?;

		Adapter::KgpOld.image_hide()?;
		Adapter::shown_store(area);
		Emulator::move_lock((area.x, area.y), |w| {
			w.write_all(&b)?;
			Ok(area)
		})
	}

	#[inline]
	pub(crate) fn image_erase(_: Rect) -> Result<()> {
		let mut w = TTY.lockout();
		write!(w, "{START}_Gq=2,a=d,d=A{ESCAPE}\\{CLOSE}")?;
		w.flush()?;
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
					"{START}_Gq=2,a=T,z=-1,C=1,f={format},s={},v={},m={};{}{ESCAPE}\\{CLOSE}",
					size.0,
					size.1,
					it.peek().is_some() as u8,
					unsafe { str::from_utf8_unchecked(first) },
				)?;
			}

			while let Some(chunk) = it.next() {
				write!(buf, "{START}_Gm={};{}{ESCAPE}\\{CLOSE}", it.peek().is_some() as u8, unsafe {
					str::from_utf8_unchecked(chunk)
				})?;
			}

			write!(buf, "{CLOSE}")?;
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
