use std::{io::Write, path::Path};

use anyhow::{bail, Result};
use color_quant::NeuQuant;
use image::DynamicImage;
use ratatui::layout::Rect;
use yazi_config::PREVIEW;
use yazi_shared::term::Term;

use crate::{adaptor::Adaptor, Image, CLOSE, ESCAPE, START};

pub(super) struct Sixel;

impl Sixel {
	pub(super) async fn image_show(path: &Path, rect: Rect) -> Result<(u32, u32)> {
		let img = Image::downscale(path, rect).await?;
		let size = (img.width(), img.height());
		let b = Self::encode(img).await?;

		Adaptor::Sixel.image_hide()?;
		Adaptor::shown_store(rect, size);
		Term::move_lock((rect.x, rect.y), |stderr| {
			stderr.write_all(&b)?;
			Ok(size)
		})
	}

	pub(super) fn image_erase(rect: Rect) -> Result<()> {
		let s = " ".repeat(rect.width as usize);
		Term::move_lock((0, 0), |stderr| {
			for y in rect.top()..rect.bottom() {
				Term::move_to(stderr, rect.x, y)?;
				write!(stderr, "{s}")?;
			}
			Ok(())
		})
	}

	async fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		let alpha = img.color().has_alpha();
		if img.width() == 0 || img.height() == 0 {
			bail!("image is empty");
		}

		tokio::task::spawn_blocking(move || {
			let img = img.into_rgba8();
			let nq = NeuQuant::new(PREVIEW.sixel_fraction as i32, 256 - alpha as usize, &img);

			let mut buf: Vec<u8> = Vec::with_capacity(1 << 16);
			write!(buf, "{}P0;1;8q\"1;1;{};{}", START, img.width(), img.height())?;

			// Palette
			for (i, c) in nq.color_map_rgba().chunks(4).enumerate() {
				write!(
					buf,
					"#{};2;{};{};{}",
					i + alpha as usize,
					c[0] as u16 * 100 / 255,
					c[1] as u16 * 100 / 255,
					c[2] as u16 * 100 / 255
				)?;
			}

			for y in 0..img.height() {
				let c = (b'?' + (1 << (y % 6))) as char;

				let mut last = 0;
				let mut repeat = 0usize;
				for x in 0..img.width() {
					let pixel = img.get_pixel(x, y).0;
					let idx = if pixel[3] == 0 { 0 } else { nq.index_of(&pixel) as u8 + alpha as u8 };

					if idx == last || repeat == 0 {
						(last, repeat) = (idx, repeat + 1);
						continue;
					}

					if repeat > 1 {
						write!(buf, "#{last}!{repeat}{c}")?;
					} else {
						write!(buf, "#{last}{c}")?;
					}

					(last, repeat) = (idx, 1);
				}

				if repeat > 1 {
					write!(buf, "#{last}!{repeat}{c}")?;
				} else {
					write!(buf, "#{last}{c}")?;
				}

				write!(buf, "$")?;
				if y % 6 == 5 {
					write!(buf, "-")?;
				}
			}

			write!(buf, "{}\\{}", ESCAPE, CLOSE)?;
			Ok(buf)
		})
		.await?
	}
}
