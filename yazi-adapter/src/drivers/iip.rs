use std::{fmt::Write, io::Write as ioWrite, path::Path};

use anyhow::Result;
use base64::{Engine, engine::{Config, general_purpose::STANDARD}};
use crossterm::{cursor::MoveTo, queue};
use image::{DynamicImage, ExtendedColorType, ImageEncoder, codecs::{jpeg::JpegEncoder, png::PngEncoder}};
use ratatui::layout::Rect;
use yazi_config::YAZI;

use crate::{CLOSE, Emulator, Image, START, adapter::Adapter};

pub(crate) struct Iip;

impl Iip {
	pub(crate) async fn image_show(path: &Path, max: Rect) -> Result<Rect> {
		let img = Image::downscale(path, max).await?;
		let area = Image::pixel_area((img.width(), img.height()), max);
		let b = Self::encode(img).await?;

		Adapter::Iip.image_hide()?;
		Adapter::shown_store(area);
		Emulator::move_lock((max.x, max.y), |w| {
			w.write_all(&b)?;
			Ok(area)
		})
	}

	pub(crate) fn image_erase(area: Rect) -> Result<()> {
		let s = " ".repeat(area.width as usize);
		Emulator::move_lock((0, 0), |w| {
			for y in area.top()..area.bottom() {
				queue!(w, MoveTo(area.x, y))?;
				write!(w, "{s}")?;
			}
			Ok(())
		})
	}

	async fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		tokio::task::spawn_blocking(move || {
			let (w, h) = (img.width(), img.height());

			let mut b = vec![];
			if img.color().has_alpha() {
				PngEncoder::new(&mut b).write_image(&img.into_rgba8(), w, h, ExtendedColorType::Rgba8)?;
			} else {
				JpegEncoder::new_with_quality(&mut b, YAZI.preview.image_quality).encode_image(&img)?;
			};

			let mut buf = String::with_capacity(
				200 + base64::encoded_len(b.len(), STANDARD.config().encode_padding()).unwrap_or(0),
			);

			write!(
				buf,
				"{START}]1337;File=inline=1;size={};width={w}px;height={h}px;doNotMoveCursor=1:",
				b.len(),
			)?;
			STANDARD.encode_string(b, &mut buf);
			write!(buf, "\x07{CLOSE}")?;

			Ok(buf.into_bytes())
		})
		.await?
	}
}
