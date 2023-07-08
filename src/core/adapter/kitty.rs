use std::io::Write;

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use image::DynamicImage;

pub struct Kitty;

impl Kitty {
	pub fn image_show(img: DynamicImage) -> Result<Vec<u8>> {
		fn output(raw: Vec<u8>, format: u8, size: (u32, u32)) -> Result<Vec<u8>> {
			let b64 = general_purpose::STANDARD.encode(raw).chars().collect::<Vec<_>>();

			let mut it = b64.chunks(4096).peekable();
			let mut buf = Vec::with_capacity(b64.len() + it.len() * 50);
			if let Some(first) = it.next() {
				write!(
					buf,
					"\x1b_Ga=d\x1b\\\x1b_Ga=T,f={},s={},v={},m={};{}\x1b\\",
					format,
					size.0,
					size.1,
					it.peek().is_some() as u8,
					first.iter().collect::<String>(),
				)?;
			}

			while let Some(chunk) = it.next() {
				write!(
					buf,
					"\x1b_Gm={};{}\x1b\\",
					it.peek().is_some() as u8,
					chunk.iter().collect::<String>()
				)?;
			}
			Ok(buf)
		}

		let size = (img.width(), img.height());
		match img {
			DynamicImage::ImageRgb8(v) => output(v.into_raw(), 24, size),
			DynamicImage::ImageRgba8(v) => output(v.into_raw(), 32, size),
			v => output(v.to_rgb8().into_raw(), 24, size),
		}
	}

	#[inline]
	pub fn image_hide() -> &'static [u8; 8] { b"\x1b_Ga=d\x1b\\" }
}
