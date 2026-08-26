use core::str;
use std::{io::Write, path::PathBuf};

use anyhow::Result;
use base64::{Engine, engine::general_purpose};
use image::DynamicImage;
use ratatui_core::layout::Rect;
use yazi_emulator::{CLOSE, ESCAPE, Emulator, START};
use yazi_macro::writef;
use yazi_tty::{TTY, sequence::MoveTo};

use crate::{ADAPTOR, Image, drivers::Kgp};

pub(super) struct KgpOld;

impl KgpOld {
	pub(super) async fn image_show(path: PathBuf, max: Rect) -> Result<Rect> {
		let img = Image::downscale(path, max).await?;
		let size = (img.width(), img.height());
		let area = Image::pixel_area(size, max);

		let b1 = Self::encode(img, size).await?;
		let b2 = Self::place(area, size)?;

		ADAPTOR.image_hide()?;
		ADAPTOR.shown_store(area);
		Emulator::move_lock((area.x, area.y), |w| {
			w.write_all(&b1)?;
			w.write_all(&b2)?;
			Ok(area)
		})
	}

	pub(super) fn image_erase(area: Rect) -> Result<()> {
		let mut w = TTY.lockout();
		let Some(shown) = ADAPTOR.shown_area() else {
			writef!(w, "{START}_Gq=2,a=d,d=I,i={}{ESCAPE}\\{CLOSE}", Kgp::image_id())?;
			return Ok(());
		};

		for y in area.top()..area.bottom() {
			for x in area.left()..area.right() {
				let p = (y - shown.y) as u32 * shown.width as u32 + (x - shown.x) as u32 + 1;
				write!(w, "{START}_Gq=2,a=d,d=i,i={},p={p}{ESCAPE}\\{CLOSE}", Kgp::image_id())?;
			}
		}
		w.flush()?;
		Ok(())
	}

	async fn encode(img: DynamicImage, size: (u32, u32)) -> Result<Vec<u8>> {
		fn output(raw: &[u8], format: u8, (w, h): (u32, u32)) -> Result<Vec<u8>> {
			let b64 = general_purpose::STANDARD.encode(raw).into_bytes();

			let mut it = b64.chunks(4096).peekable();
			let mut buf = Vec::with_capacity(b64.len() + it.len() * 50);
			if let Some(first) = it.next() {
				write!(
					buf,
					"{START}_Gq=2,a=t,i={},f={format},s={w},v={h},m={};{}{ESCAPE}\\{CLOSE}",
					Kgp::image_id(),
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

		tokio::task::spawn_blocking(move || match img {
			DynamicImage::ImageRgb8(v) => output(v.as_raw(), 24, size),
			DynamicImage::ImageRgba8(v) => output(v.as_raw(), 32, size),
			v => output(v.into_rgb8().as_raw(), 24, size),
		})
		.await?
	}

	fn place(area: Rect, (w, h): (u32, u32)) -> Result<Vec<u8>> {
		let mut buf = Vec::with_capacity(area.width as usize * area.height as usize * 100);
		let (cols, rows) = (area.width as u32, area.height as u32);

		for y in 0..area.height as u32 {
			let top = h * y / rows;
			let bottom = (h * (y + 1) / rows).max(top + 1);
			for x in 0..area.width as u32 {
				let left = w * x / cols;
				let right = (w * (x + 1) / cols).max(left + 1);

				let p = y * cols + x + 1;
				write!(
					buf,
					"{}{START}_Gq=2,a=p,i={},p={p},x={left},y={top},w={},h={},c=1,r=1,z=-1,C=1{ESCAPE}\\{CLOSE}",
					MoveTo(area.x + x as u16, area.y + y as u16),
					Kgp::image_id(),
					right - left,
					bottom - top,
				)?;
			}
		}

		Ok(buf)
	}
}
