use std::{io::Write, path::Path};

use anyhow::{Result, bail};
use color_quant::NeuQuant;
use crossterm::{cursor::MoveTo, queue};
use image::DynamicImage;
use ratatui::layout::Rect;
use yazi_config::YAZI;

use crate::{CLOSE, ESCAPE, Emulator, Image, START, adapter::Adapter};

pub(crate) struct Sixel;

impl Sixel {
	pub(crate) async fn image_show(path: &Path, max: Rect) -> Result<Rect> {
		let img = Image::downscale(path, max).await?;
		let area = Image::pixel_area((img.width(), img.height()), max);
		let b = Self::encode(img).await?;

		Adapter::Sixel.image_hide()?;
		Adapter::shown_store(area);
		Emulator::move_lock((area.x, area.y), |w| {
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
		let alpha = img.color().has_alpha();
		if img.width() == 0 || img.height() == 0 {
			bail!("image is empty");
		}

		tokio::task::spawn_blocking(move || {
			let img = img.into_rgba8();
			let nq = NeuQuant::new(YAZI.preview.sixel_fraction as i32, 256 - alpha as usize, &img);

			let mut buf: Vec<u8> = Vec::with_capacity(1 << 16);
			write!(buf, "{START}P0;1;8q\"1;1;{};{}", img.width(), img.height())?;

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

			write!(buf, "{ESCAPE}\\{CLOSE}")?;
			Ok(buf)
		})
		.await?
	}
}
