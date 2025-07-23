use std::io::Write;

use anyhow::{Result, bail};
use crossterm::{cursor::MoveTo, queue};
use image::{DynamicImage, GenericImageView, RgbImage};
use palette::{Srgb, cast::ComponentsAs};
use quantette::{ColorSlice, PaletteSize, QuantizeOutput, wu::UIntBinner};
use ratatui::layout::Rect;
use yazi_shared::url::Url;

use crate::{CLOSE, ESCAPE, Emulator, Image, START, adapter::Adapter};

pub(crate) struct Sixel;

impl Sixel {
	pub(crate) async fn image_show(url: &Url, max: Rect) -> Result<Rect> {
		let img = Image::downscale(url, max).await?;
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

		let (qo, img) = tokio::task::spawn_blocking(move || match &img {
			DynamicImage::ImageRgb8(rgb) => Self::quantify(rgb, false).map(|q| (q, img)),
			_ => Self::quantify(&img.to_rgb8(), alpha).map(|q| (q, img)),
		})
		.await??;

		tokio::task::spawn_blocking(move || {
			let mut buf = vec![];
			write!(buf, "{START}P0;1;8q\"1;1;{};{}", img.width(), img.height())?;

			// Palette
			for (i, c) in qo.palette.iter().enumerate() {
				write!(
					buf,
					"#{};2;{};{};{}",
					i + alpha as usize,
					c.red as u16 * 100 / 255,
					c.green as u16 * 100 / 255,
					c.blue as u16 * 100 / 255
				)?;
			}

			for y in 0..img.height() {
				let c = (b'?' + (1 << (y % 6))) as char;

				let mut last = 0;
				let mut repeat = 0usize;
				for x in 0..img.width() {
					let idx = if img.get_pixel(x, y)[3] == 0 {
						0
					} else {
						qo.indices[y as usize * img.width() as usize + x as usize] + alpha as u8
					};

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

	fn quantify(rgb: &RgbImage, alpha: bool) -> Result<QuantizeOutput<Srgb<u8>>> {
		let buf = &rgb.as_raw()[..(rgb.pixels().len() * 3)];
		let slice: ColorSlice<Srgb<u8>> = buf.components_as().try_into()?;

		Ok(quantette::wu::indexed_palette(
			&slice,
			PaletteSize::try_from(256u16 - alpha as u16)?,
			&UIntBinner::<32>,
		))
	}
}
