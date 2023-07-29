use std::{io::Write, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::{codecs::jpeg::JpegEncoder, DynamicImage};
use ratatui::prelude::Rect;
use tokio::io::AsyncWriteExt;

use super::image::Image;
use crate::ui::Term;

pub(super) struct Iterm2;

impl Iterm2 {
	pub(super) async fn image_show(path: &Path, rect: Rect) -> Result<()> {
		let img = Image::resize(path, (rect.width, rect.height)).await?;
		let b = Self::encode(img).await?;

		Term::move_to(rect.x, rect.y).ok();
		tokio::io::stdout().write_all(&b).await.ok();
		Ok(())
	}

	#[inline]
	pub(super) fn image_hide(rect: Rect) {
		let s = " ".repeat(rect.width as usize);
		for y in rect.top()..=rect.bottom() {
			Term::move_to(rect.x, y).ok();
			std::io::stdout().write_all(s.as_bytes()).ok();
		}
	}

	async fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		tokio::task::spawn_blocking(move || {
			let size = (img.width(), img.height());

			let mut jpg = vec![];
			JpegEncoder::new_with_quality(&mut jpg, 75).encode_image(&img)?;

			let mut buf = vec![];
			write!(
				buf,
				"\x1b]1337;File=inline=1;size={};width={}px;height={}px:{}\x07",
				jpg.len(),
				size.0,
				size.1,
				general_purpose::STANDARD.encode(&jpg)
			)?;
			Ok(buf)
		})
		.await?
	}
}
