use std::{fmt::Write, io::Write as ioWrite, path::Path};

use anyhow::Result;
use base64::{engine::{general_purpose::STANDARD, Config}, Engine};
use crossterm::{cursor::MoveTo, queue};
use image::{codecs::{jpeg::JpegEncoder, png::PngEncoder}, DynamicImage, ExtendedColorType, ImageEncoder};
use ratatui::layout::Rect;
use yazi_config::PREVIEW;

use super::image::Image;
use crate::{adapter::Adapter, Emulator, CLOSE, START};

pub(super) struct Iip;

impl Iip {
	pub(super) async fn image_show(path: &Path, max: Rect) -> Result<Rect> {
		let img = Image::downscale(path, max).await?;
		let area = Image::pixel_area((img.width(), img.height()), max);
		let b = Self::encode(img).await?;

		Adapter::Iip.image_hide()?;
		Adapter::shown_store(area);
		Emulator::move_lock((max.x, max.y), |stderr| {
			stderr.write_all(&b)?;
			Ok(area)
		})
	}

	pub(super) fn image_erase(area: Rect) -> Result<()> {
		let s = " ".repeat(area.width as usize);
		Emulator::move_lock((0, 0), |stderr| {
			for y in area.top()..area.bottom() {
				queue!(stderr, MoveTo(area.x, y))?;
				write!(stderr, "{s}")?;
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
				JpegEncoder::new_with_quality(&mut b, PREVIEW.image_quality).encode_image(&img)?;
			};

			let mut buf = String::with_capacity(
				200 + base64::encoded_len(b.len(), STANDARD.config().encode_padding()).unwrap_or(0),
			);

			write!(
				buf,
				"{}]1337;File=inline=1;size={};width={}px;height={}px;doNotMoveCursor=1:",
				START,
				b.len(),
				w,
				h,
			)?;
			STANDARD.encode_string(b, &mut buf);
			write!(buf, "\x07{}", CLOSE)?;

			Ok(buf.into_bytes())
		})
		.await?
	}
}
