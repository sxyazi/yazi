use std::{io::Write, path::Path};

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::{codecs::jpeg::JpegEncoder, DynamicImage};
use ratatui::layout::Rect;
use yazi_shared::term::Term;

use super::image::Image;
use crate::{adaptor::Adaptor, CLOSE, START};

pub(super) struct Iterm2;

impl Iterm2 {
	pub(super) async fn image_show(path: &Path, max: Rect) -> Result<Rect> {
		let img = Image::downscale(path, max).await?;
		let area = Image::pixel_area((img.width(), img.height()), max);
		let b = Self::encode(img).await?;

		Adaptor::Iterm2.image_hide()?;
		Adaptor::shown_store(area);
		Term::move_lock((max.x, max.y), |stderr| {
			stderr.write_all(&b)?;
			Ok(area)
		})
	}

	pub(super) fn image_erase(area: Rect) -> Result<()> {
		let s = " ".repeat(area.width as usize);
		Term::move_lock((0, 0), |stderr| {
			for y in area.top()..area.bottom() {
				Term::move_to(stderr, area.x, y)?;
				write!(stderr, "{s}")?;
			}
			Ok(())
		})
	}

	async fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		tokio::task::spawn_blocking(move || {
			let size = (img.width(), img.height());

			let mut jpg = vec![];
			JpegEncoder::new_with_quality(&mut jpg, 75).encode_image(&img)?;

			let mut buf = vec![];
			write!(
				buf,
				"{}]1337;File=inline=1;size={};width={}px;height={}px;doNotMoveCursor=1:{}\x07{}",
				START,
				jpg.len(),
				size.0,
				size.1,
				general_purpose::STANDARD.encode(&jpg),
				CLOSE
			)?;
			Ok(buf)
		})
		.await?
	}
}
