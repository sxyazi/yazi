use std::{io::Write, path::Path};

use anyhow::Result;
use base64::{engine::{general_purpose::STANDARD, Config}, Engine};
use crossterm::{cursor::MoveTo, queue};
use image::{codecs::jpeg::JpegEncoder, DynamicImage};
use ratatui::layout::Rect;

use super::image::Image;
use crate::{adapter::Adapter, Emulator, CLOSE, START};

pub(super) struct Iterm2;

impl Iterm2 {
	pub(super) async fn image_show(path: &Path, max: Rect) -> Result<Rect> {
		let img = Image::downscale(path, max).await?;
		let area = Image::pixel_area((img.width(), img.height()), max);
		let b = Self::encode(img).await?;

		Adapter::Iterm2.image_hide()?;
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
			let mut jpg = vec![];
			JpegEncoder::new_with_quality(&mut jpg, 75).encode_image(&img)?;

			let len = base64::encoded_len(jpg.len(), STANDARD.config().encode_padding());
			let mut buf = Vec::with_capacity(200 + len.unwrap_or(1 << 16));

			write!(
				buf,
				"{}]1337;File=inline=1;size={};width={}px;height={}px;doNotMoveCursor=1:{}\x07{}",
				START,
				jpg.len(),
				img.width(),
				img.height(),
				STANDARD.encode(&jpg),
				CLOSE
			)?;
			Ok(buf)
		})
		.await?
	}
}
