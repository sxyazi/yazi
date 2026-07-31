use std::str;

use compact_str::CompactString;

use crate::{ParseError, Result, bail, event::{Event, Report}, parser::Parser};

impl Parser {
	pub(super) fn parse_csi_report(&self) -> Result<Event> {
		let seq = &self.seq;
		debug_assert!(seq.starts_with(b"\x1b["));

		Ok(Event::Report(match (seq.get(2), seq.last()) {
			// `CSI ? Ps ; ... c` (`\x1b[?...c`) - DA1 (Primary Device Attributes) response.
			(Some(b'?'), Some(b'c')) => Report::Da1(
				str::from_utf8(&seq[3..seq.len() - 1])?
					.split(';')
					.filter(|s| !s.is_empty())
					.map(str::parse)
					.collect::<Result<_, _>>()?,
			),

			// `CSI ? 12 ; Ps $ y` (`\x1b[?12;...$y`) - DECRPM response for DEC mode 12.
			(Some(b'?'), Some(b'y')) => {
				let status: u8 = str::from_utf8(&seq[3..seq.len() - 1])?
					.strip_prefix("12;")
					.and_then(|s| s.strip_suffix('$'))
					.ok_or(ParseError::Invalid)?
					.parse()?;

				Report::CursorBlink(match status {
					1 | 3 => true,
					2 | 4 => false,
					_ => bail!(),
				})
			}

			// `CSI 6 ; Ph ; Pw t` (`\x1b[6;...;...t`) - XTWINOPS character-cell size report.
			(Some(b'6'), Some(b't')) => {
				let (h, w) = str::from_utf8(&seq[3..seq.len() - 1])?
					.strip_prefix(';')
					.and_then(|s| s.split_once(';'))
					.ok_or(ParseError::Invalid)?;
				Report::CellPixelSize { width: w.parse()?, height: h.parse()? }
			}

			_ => bail!(),
		}))
	}

	pub(super) fn parse_dcs_report(&self) -> Result<Event> {
		let seq = self.seq.strip_suffix(b"\x1b\\").ok_or(ParseError::Invalid)?;

		// DECRQSS response for DECSCUSR:
		//   `DCS 1 $ r Ps SP q ST` (`\x1bP1$r... q\x1b\\`)
		if let Some(body) = seq.strip_prefix(b"\x1bP1$r") {
			return Ok(Event::Report(match body {
				[shape @ b'0'..=b'6', b' ', b'q'] => Report::CursorShape(shape - b'0'),
				_ => bail!(),
			}));
		}

		// `DCS > | Pt ST` (`\x1bP>|...\x1b\\`) - XTVERSION response.
		if let Some(body) = seq.strip_prefix(b"\x1bP>|") {
			return Ok(Event::Report(Report::XtVersion(CompactString::from_utf8_lossy(body))));
		}

		bail!()
	}

	pub(super) fn parse_osc_report(&self) -> Result<Event> {
		let seq = self
			.seq
			.strip_suffix(b"\x07")
			.or_else(|| self.seq.strip_suffix(b"\x1b\\"))
			.ok_or(ParseError::Invalid)?;

		// Dynamic background-color report:
		//   `OSC 11 ; rgb:Pr/Pg/Pb ST|BEL` (`\x1b]11;rgb:...`)
		if let Some(body) = seq.strip_prefix(b"\x1b]11;rgb:") {
			return Ok(Event::Report(Report::BackgroundColor(parse_bg_rgb(body)?)));
		}

		bail!()
	}

	pub(super) fn parse_apc_report(&self) -> Result<Event> {
		let seq = self.seq.strip_suffix(b"\x1b\\").ok_or(ParseError::Invalid)?;

		// Kitty graphics response:
		//   `APC G i=Pi,...;OK|ERROR ST` (`\x1b_Gi=...;...\x1b\\`)
		if let Some(body) = seq.strip_prefix(b"\x1b_G") {
			let (meta, status) = str::from_utf8(body)?.split_once(';').ok_or(ParseError::Invalid)?;
			let id =
				meta.split(',').find_map(|s| s.strip_prefix("i=")).ok_or(ParseError::Invalid)?.parse()?;
			return Ok(Event::Report(Report::KittyGraphics { id, ok: status == "OK" }));
		}

		bail!()
	}
}

fn parse_bg_rgb(body: &[u8]) -> Result<[u16; 3]> {
	let mut it = str::from_utf8(body)?.split('/');
	let mut rgb = [0; 3];

	for value in &mut rgb {
		let s = it.next().ok_or(ParseError::Invalid)?;
		let n = u16::from_str_radix(s, 16)?;
		*value = match s.len() {
			1 => n * 0x1111,          // A    -> AAAA
			2 => n * 0x0101,          // AB   -> ABAB
			3 => (n << 4) | (n >> 8), // ABC  -> ABCA
			4 => n,                   // ABCD -> ABCD
			_ => bail!(),
		};
	}

	if it.next().is_some() { bail!() } else { Ok(rgb) }
}
