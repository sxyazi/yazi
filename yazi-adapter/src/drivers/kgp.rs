use core::str;
use std::io::Write;

use anyhow::Result;
use base64::{Engine, engine::general_purpose};
use crossterm::{cursor::MoveTo, queue};
use image::DynamicImage;
use ratatui::layout::Rect;
use yazi_shared::url::Url;

use crate::{CLOSE, ESCAPE, Emulator, START, adapter::Adapter, image::Image};

static DIACRITICS: [char; 297] = [
	'\u{0305}',
	'\u{030D}',
	'\u{030E}',
	'\u{0310}',
	'\u{0312}',
	'\u{033D}',
	'\u{033E}',
	'\u{033F}',
	'\u{0346}',
	'\u{034A}',
	'\u{034B}',
	'\u{034C}',
	'\u{0350}',
	'\u{0351}',
	'\u{0352}',
	'\u{0357}',
	'\u{035B}',
	'\u{0363}',
	'\u{0364}',
	'\u{0365}',
	'\u{0366}',
	'\u{0367}',
	'\u{0368}',
	'\u{0369}',
	'\u{036A}',
	'\u{036B}',
	'\u{036C}',
	'\u{036D}',
	'\u{036E}',
	'\u{036F}',
	'\u{0483}',
	'\u{0484}',
	'\u{0485}',
	'\u{0486}',
	'\u{0487}',
	'\u{0592}',
	'\u{0593}',
	'\u{0594}',
	'\u{0595}',
	'\u{0597}',
	'\u{0598}',
	'\u{0599}',
	'\u{059C}',
	'\u{059D}',
	'\u{059E}',
	'\u{059F}',
	'\u{05A0}',
	'\u{05A1}',
	'\u{05A8}',
	'\u{05A9}',
	'\u{05AB}',
	'\u{05AC}',
	'\u{05AF}',
	'\u{05C4}',
	'\u{0610}',
	'\u{0611}',
	'\u{0612}',
	'\u{0613}',
	'\u{0614}',
	'\u{0615}',
	'\u{0616}',
	'\u{0617}',
	'\u{0657}',
	'\u{0658}',
	'\u{0659}',
	'\u{065A}',
	'\u{065B}',
	'\u{065D}',
	'\u{065E}',
	'\u{06D6}',
	'\u{06D7}',
	'\u{06D8}',
	'\u{06D9}',
	'\u{06DA}',
	'\u{06DB}',
	'\u{06DC}',
	'\u{06DF}',
	'\u{06E0}',
	'\u{06E1}',
	'\u{06E2}',
	'\u{06E4}',
	'\u{06E7}',
	'\u{06E8}',
	'\u{06EB}',
	'\u{06EC}',
	'\u{0730}',
	'\u{0732}',
	'\u{0733}',
	'\u{0735}',
	'\u{0736}',
	'\u{073A}',
	'\u{073D}',
	'\u{073F}',
	'\u{0740}',
	'\u{0741}',
	'\u{0743}',
	'\u{0745}',
	'\u{0747}',
	'\u{0749}',
	'\u{074A}',
	'\u{07EB}',
	'\u{07EC}',
	'\u{07ED}',
	'\u{07EE}',
	'\u{07EF}',
	'\u{07F0}',
	'\u{07F1}',
	'\u{07F3}',
	'\u{0816}',
	'\u{0817}',
	'\u{0818}',
	'\u{0819}',
	'\u{081B}',
	'\u{081C}',
	'\u{081D}',
	'\u{081E}',
	'\u{081F}',
	'\u{0820}',
	'\u{0821}',
	'\u{0822}',
	'\u{0823}',
	'\u{0825}',
	'\u{0826}',
	'\u{0827}',
	'\u{0829}',
	'\u{082A}',
	'\u{082B}',
	'\u{082C}',
	'\u{082D}',
	'\u{0951}',
	'\u{0953}',
	'\u{0954}',
	'\u{0F82}',
	'\u{0F83}',
	'\u{0F86}',
	'\u{0F87}',
	'\u{135D}',
	'\u{135E}',
	'\u{135F}',
	'\u{17DD}',
	'\u{193A}',
	'\u{1A17}',
	'\u{1A75}',
	'\u{1A76}',
	'\u{1A77}',
	'\u{1A78}',
	'\u{1A79}',
	'\u{1A7A}',
	'\u{1A7B}',
	'\u{1A7C}',
	'\u{1B6B}',
	'\u{1B6D}',
	'\u{1B6E}',
	'\u{1B6F}',
	'\u{1B70}',
	'\u{1B71}',
	'\u{1B72}',
	'\u{1B73}',
	'\u{1CD0}',
	'\u{1CD1}',
	'\u{1CD2}',
	'\u{1CDA}',
	'\u{1CDB}',
	'\u{1CE0}',
	'\u{1DC0}',
	'\u{1DC1}',
	'\u{1DC3}',
	'\u{1DC4}',
	'\u{1DC5}',
	'\u{1DC6}',
	'\u{1DC7}',
	'\u{1DC8}',
	'\u{1DC9}',
	'\u{1DCB}',
	'\u{1DCC}',
	'\u{1DD1}',
	'\u{1DD2}',
	'\u{1DD3}',
	'\u{1DD4}',
	'\u{1DD5}',
	'\u{1DD6}',
	'\u{1DD7}',
	'\u{1DD8}',
	'\u{1DD9}',
	'\u{1DDA}',
	'\u{1DDB}',
	'\u{1DDC}',
	'\u{1DDD}',
	'\u{1DDE}',
	'\u{1DDF}',
	'\u{1DE0}',
	'\u{1DE1}',
	'\u{1DE2}',
	'\u{1DE3}',
	'\u{1DE4}',
	'\u{1DE5}',
	'\u{1DE6}',
	'\u{1DFE}',
	'\u{20D0}',
	'\u{20D1}',
	'\u{20D4}',
	'\u{20D5}',
	'\u{20D6}',
	'\u{20D7}',
	'\u{20DB}',
	'\u{20DC}',
	'\u{20E1}',
	'\u{20E7}',
	'\u{20E9}',
	'\u{20F0}',
	'\u{2CEF}',
	'\u{2CF0}',
	'\u{2CF1}',
	'\u{2DE0}',
	'\u{2DE1}',
	'\u{2DE2}',
	'\u{2DE3}',
	'\u{2DE4}',
	'\u{2DE5}',
	'\u{2DE6}',
	'\u{2DE7}',
	'\u{2DE8}',
	'\u{2DE9}',
	'\u{2DEA}',
	'\u{2DEB}',
	'\u{2DEC}',
	'\u{2DED}',
	'\u{2DEE}',
	'\u{2DEF}',
	'\u{2DF0}',
	'\u{2DF1}',
	'\u{2DF2}',
	'\u{2DF3}',
	'\u{2DF4}',
	'\u{2DF5}',
	'\u{2DF6}',
	'\u{2DF7}',
	'\u{2DF8}',
	'\u{2DF9}',
	'\u{2DFA}',
	'\u{2DFB}',
	'\u{2DFC}',
	'\u{2DFD}',
	'\u{2DFE}',
	'\u{2DFF}',
	'\u{A66F}',
	'\u{A67C}',
	'\u{A67D}',
	'\u{A6F0}',
	'\u{A6F1}',
	'\u{A8E0}',
	'\u{A8E1}',
	'\u{A8E2}',
	'\u{A8E3}',
	'\u{A8E4}',
	'\u{A8E5}',
	'\u{A8E6}',
	'\u{A8E7}',
	'\u{A8E8}',
	'\u{A8E9}',
	'\u{A8EA}',
	'\u{A8EB}',
	'\u{A8EC}',
	'\u{A8ED}',
	'\u{A8EE}',
	'\u{A8EF}',
	'\u{A8F0}',
	'\u{A8F1}',
	'\u{AAB0}',
	'\u{AAB2}',
	'\u{AAB3}',
	'\u{AAB7}',
	'\u{AAB8}',
	'\u{AABE}',
	'\u{AABF}',
	'\u{AAC1}',
	'\u{FE20}',
	'\u{FE21}',
	'\u{FE22}',
	'\u{FE23}',
	'\u{FE24}',
	'\u{FE25}',
	'\u{FE26}',
	'\u{10A0F}',
	'\u{10A38}',
	'\u{1D185}',
	'\u{1D186}',
	'\u{1D187}',
	'\u{1D188}',
	'\u{1D189}',
	'\u{1D1AA}',
	'\u{1D1AB}',
	'\u{1D1AC}',
	'\u{1D1AD}',
	'\u{1D242}',
	'\u{1D243}',
	'\u{1D244}',
];

pub(crate) struct Kgp;

impl Kgp {
	pub(crate) async fn image_show(url: &Url, max: Rect) -> Result<Rect> {
		let img = Image::downscale(url, max).await?;
		let area = Image::pixel_area((img.width(), img.height()), max);

		let b1 = Self::encode(img).await?;
		let b2 = Self::place(&area)?;

		Adapter::Kgp.image_hide()?;
		Adapter::shown_store(area);
		Emulator::move_lock((area.x, area.y), |w| {
			w.write_all(&b1)?;
			w.write_all(&b2)?;
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

			write!(w, "{START}_Gq=2,a=d,d=A{ESCAPE}\\{CLOSE}")?;
			Ok(())
		})
	}

	async fn encode(img: DynamicImage) -> Result<Vec<u8>> {
		fn output(raw: &[u8], format: u8, size: (u32, u32)) -> Result<Vec<u8>> {
			let b64 = general_purpose::STANDARD.encode(raw).into_bytes();

			let mut it = b64.chunks(4096).peekable();
			let mut buf = Vec::with_capacity(b64.len() + it.len() * 50);
			if let Some(first) = it.next() {
				write!(
					buf,
					"{START}_Gq=2,a=T,i=1,C=1,U=1,f={format},s={},v={},m={};{}{ESCAPE}\\{CLOSE}",
					size.0,
					size.1,
					it.peek().is_some() as u8,
					unsafe { str::from_utf8_unchecked(first) },
				)?;
			}

			while let Some(chunk) = it.next() {
				write!(buf, "{START}_Gm={};{}{ESCAPE}\\{CLOSE}", it.peek().is_some() as u8, unsafe {
					str::from_utf8_unchecked(chunk)
				})?;
			}

			write!(buf, "{CLOSE}")?;
			Ok(buf)
		}

		let size = (img.width(), img.height());
		tokio::task::spawn_blocking(move || match img {
			DynamicImage::ImageRgb8(v) => output(v.as_raw(), 24, size),
			DynamicImage::ImageRgba8(v) => output(v.as_raw(), 32, size),
			v => output(v.into_rgb8().as_raw(), 24, size),
		})
		.await?
	}

	fn place(area: &Rect) -> Result<Vec<u8>> {
		let mut buf = Vec::with_capacity(area.width as usize * area.height as usize * 3 + 50);
		write!(buf, "\x1b[38;2;0;0;1m")?;
		for y in 0..area.height {
			write!(buf, "\x1b[{};{}H", area.y + y + 1, area.x + 1)?;
			for x in 0..area.width {
				write!(buf, "\u{10EEEE}")?;
				write!(buf, "{}", *DIACRITICS.get(y as usize).unwrap_or(&DIACRITICS[0]))?;
				write!(buf, "{}", *DIACRITICS.get(x as usize).unwrap_or(&DIACRITICS[0]))?;
			}
		}
		Ok(buf)
	}
}
