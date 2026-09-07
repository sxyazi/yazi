use std::{borrow::Cow, fmt::{self, Display, Write}};

use anyhow::{Result, anyhow, bail};

use super::writer::TwiddleWriter;

pub(crate) struct Twiddle;

impl Twiddle {
	pub(crate) fn encode(writer: &mut impl fmt::Write, value: &impl Display) -> fmt::Result {
		write!(&mut TwiddleWriter(writer), "{value}")
	}

	pub(crate) fn encode_str(writer: &mut impl fmt::Write, value: &str) -> fmt::Result {
		TwiddleWriter(writer).write_str(value)
	}

	#[inline]
	pub(super) fn encode_byte<W: fmt::Write + ?Sized>(writer: &mut W, byte: u8) -> fmt::Result {
		const HEX: &[u8; 16] = b"0123456789ABCDEF";

		match byte {
			b'/' => writer.write_str("~s"),
			b'%' => writer.write_str("~p"),
			b'~' => writer.write_str("~~"),
			_ => {
				writer.write_char('~')?;
				writer.write_char(HEX[(byte >> 4) as usize] as char)?;
				writer.write_char(HEX[(byte & 0xf) as usize] as char)
			}
		}
	}

	pub(crate) fn encode_bytes(writer: &mut impl fmt::Write, value: &[u8]) -> fmt::Result {
		for chunk in value.utf8_chunks() {
			Self::encode_str(writer, chunk.valid())?;
			for &byte in chunk.invalid() {
				Self::encode_byte(writer, byte)?;
			}
		}
		Ok(())
	}

	pub(crate) fn decode<'a>(value: &'a [u8]) -> Result<Cow<'a, [u8]>> {
		let Some(mut i) = value.iter().position(|&b| b == b'~') else {
			return Ok(Cow::Borrowed(value));
		};

		let mut decoded = Vec::with_capacity(value.len());
		decoded.extend_from_slice(&value[..i]);

		while i < value.len() {
			if value[i] != b'~' {
				decoded.push(value[i]);
				i += 1;
				continue;
			}

			let code = *value.get(i + 1).ok_or_else(|| anyhow!("truncated twiddle data"))?;
			match code {
				b'~' => {
					decoded.push(b'~');
					i += 2;
				}
				b's' => {
					decoded.push(b'/');
					i += 2;
				}
				b'p' => {
					decoded.push(b'%');
					i += 2;
				}
				_ => {
					let low = *value.get(i + 2).ok_or_else(|| anyhow!("truncated twiddle data"))?;
					decoded.push(Twiddle::hex(code)? << 4 | Twiddle::hex(low)?);
					i += 3;
				}
			}
		}

		Ok(Cow::Owned(decoded))
	}

	pub(crate) fn len(value: &impl Display) -> Result<usize, fmt::Error> {
		struct Len(usize);
		impl fmt::Write for Len {
			fn write_str(&mut self, value: &str) -> fmt::Result {
				self.0 += value.len();
				Ok(())
			}
		}

		let mut len = Len(0);
		write!(&mut TwiddleWriter(&mut len), "{value}")?;
		Ok(len.0)
	}

	fn len_str(value: &str) -> usize { value.bytes().map(Self::len_escape).sum() }

	pub(crate) fn len_bytes(value: &[u8]) -> usize {
		value.utf8_chunks().map(|c| Self::len_str(c.valid()) + c.invalid().len() * 3).sum()
	}

	#[inline]
	pub(super) fn len_escape(byte: u8) -> usize {
		match byte {
			b'/' | b'%' | b'~' => 2,
			0..=0x1f | 0x7f => 3,
			_ => 1,
		}
	}

	fn hex(byte: u8) -> Result<u8> {
		match byte {
			b'0'..=b'9' => Ok(byte - b'0'),
			b'A'..=b'F' => Ok(byte - b'A' + 10),
			_ => bail!("invalid twiddle data"),
		}
	}
}
