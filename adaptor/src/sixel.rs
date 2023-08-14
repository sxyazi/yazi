use std::{io::{stdout, BufWriter, Write}, path::Path};

use anyhow::{bail, Result};
use color_quant::NeuQuant;
use image::DynamicImage;
use ratatui::prelude::Rect;
use shared::Term;

use crate::Image;

pub(super) struct Sixel;

impl Sixel {
	pub(super) async fn image_show(path: &Path, rect: Rect) -> Result<()> {
		let img = Image::crop(path, (rect.width, rect.height)).await?;
		let b = Self::encode(img).await?;

		let mut stdout = stdout().lock();
		Term::move_to(&mut stdout, rect.x, rect.y)?;
		Ok(stdout.write_all(&b)?)
	}

	#[inline]
	pub(super) fn image_hide(rect: Rect) -> Result<()> {
		let s = " ".repeat(rect.width as usize);
		let mut stdout = BufWriter::new(stdout().lock());

		for y in rect.top()..=rect.bottom() {
			Term::move_to(&mut stdout, rect.x, y)?;
			stdout.write_all(s.as_bytes())?;
		}
		Ok(())
	}

	async fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		let alpha = img.color().has_alpha();
		if img.width() == 0 || img.height() == 0 {
			bail!("image is empty");
		}

		tokio::task::spawn_blocking(move || {
			let img = img.into_rgba8();
			let nq = NeuQuant::new(10, 256 - alpha as usize, &img);

			let mut buf: Vec<u8> = Vec::with_capacity(1 << 16);
			write!(buf, "\x1bP0;1;8q\"1;1;{};{}", img.width(), img.height())?;

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

			write!(buf, "\x1b\\")?;
			Ok(buf)
		})
		.await?
	}
}
