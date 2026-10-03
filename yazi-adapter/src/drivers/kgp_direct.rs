use std::{io::Write, path::PathBuf};

use anyhow::Result;
use base64::{Engine, engine::general_purpose};
use image::DynamicImage;
use ratatui_core::layout::Rect;
use yazi_emulator::{CLOSE, ESCAPE, Emulator, START};
use yazi_macro::writef;
use yazi_tty::TTY;

use super::KgpPayload;
use crate::{ADAPTOR, Image, drivers::kgp_id};

// Direct placement (a=T without U=1), as used by `kitten icat`. Unlike `Kgp`,
// it does not rely on unicode placeholders, which Zellij does not support.
pub(super) struct KgpDirect;

impl KgpDirect {
	pub(super) async fn image_show(path: PathBuf, max: Rect) -> Result<Rect> {
		let img = Image::downscale(path, max).await?;
		let area = Image::pixel_area((img.width(), img.height()), max);

		let b1 = Self::encode(img, area).await?;

		ADAPTOR.image_hide()?;
		ADAPTOR.shown_store(area);
		Emulator::move_lock((area.x, area.y), |w| {
			w.write_all(&b1)?;
			Ok(area)
		})
	}

	pub(super) fn image_erase(_area: Rect) -> Result<()> {
		let mut w = TTY.lockout();
		writef!(w, "{START}_Gq=2,a=d,d=I,i={}{ESCAPE}\\{CLOSE}", kgp_id())?;
		w.flush()?;
		Ok(())
	}

	async fn encode(img: DynamicImage, area: Rect) -> Result<KgpPayload> {
		tokio::task::spawn_blocking(move || {
			let (w, h) = (img.width(), img.height());
			let (format, raw) = match img {
				DynamicImage::ImageRgba8(v) => (32, v.into_raw()),
				v => (24, v.into_rgb8().into_raw()),
			};

			let b64 = general_purpose::STANDARD.encode(raw).into_bytes();
			let mut it = b64.chunks(4096).peekable();
			let mut pl = KgpPayload::new(b64.len() + it.len() * 50);
			if let Some(first) = it.next() {
				write!(
					pl,
					"{START}_Gq=2,a=T,C=1,f={format},s={w},v={h},c={},r={},i={},m={};{}{ESCAPE}\\{CLOSE}",
					area.width,
					area.height,
					kgp_id(),
					it.peek().is_some() as u8,
					unsafe { std::str::from_utf8_unchecked(first) },
				)?;
			}

			while let Some(chunk) = it.next() {
				write!(pl, "{START}_Gm={};{}{ESCAPE}\\{CLOSE}", it.peek().is_some() as u8, unsafe {
					std::str::from_utf8_unchecked(chunk)
				})?;
			}

			Ok(pl)
		})
		.await?
	}
}
