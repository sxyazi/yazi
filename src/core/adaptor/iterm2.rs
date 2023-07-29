use std::{io::Write, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::DynamicImage;
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

	async fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		fn output(raw: Vec<u8>, size: (u32, u32)) -> Result<Vec<u8>> {
			let mut buf = Vec::with_capacity(raw.len() * 4 / 3 + 4 + 200);

			write!(
				buf,
				"\x1b]1337;File=inline=1;size={};width={}px;height={}px:",
				raw.len(),
				size.0,
				size.1,
			)?;

			let len = buf.len();
			let written = general_purpose::STANDARD.encode_slice(raw, &mut buf[len..])?;
			buf.truncate(len + written);
			Ok(buf)
		}

		let size = (img.width(), img.height());
		tokio::task::spawn_blocking(move || match img {
			DynamicImage::ImageRgb8(v) => output(v.into_raw(), size),
			DynamicImage::ImageRgba8(v) => output(v.into_raw(), size),
			v => output(v.to_rgb8().into_raw(), size),
		})
		.await?
	}
}
