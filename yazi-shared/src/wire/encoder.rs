use std::fmt;

use super::{WireKey, varint::Varint, wire::Wire};
use crate::{twiddle::Twiddle, url::UrlLike};

pub(super) struct Encoder<W> {
	writer: W,
}

impl<W: fmt::Write> Encoder<W> {
	pub(super) fn new(writer: W) -> Self { Self { writer } }

	pub(super) fn encode(&mut self, value: &Wire) -> fmt::Result {
		match value {
			Wire::Nil => self.writer.write_char('n'),
			Wire::Boolean(b) => self.writer.write_char(if *b { '1' } else { '0' }),
			Wire::Integer(i) => {
				self.writer.write_char('i')?;
				Varint::write_i64(&mut self.writer, *i)
			}
			Wire::Number(n) => self.write_display('f', n.0),
			Wire::String(s) => {
				self.writer.write_char('s')?;
				self.write_sized(s)
			}
			Wire::List(l) => {
				if l.len() <= 25 {
					self.writer.write_char((b'A' + l.len() as u8) as char)?; // Use 'A'..'Z' for lists of length 0..25
				} else {
					self.writer.write_char('l')?;
					Varint::write_u64(&mut self.writer, l.len() as u64)?;
				}
				for v in l {
					self.encode(v)?;
				}
				Ok(())
			}
			Wire::Dict(d) if d.is_empty() => self.writer.write_char('!'),
			Wire::Dict(d) => {
				self.writer.write_char('d')?;
				Varint::write_u64(&mut self.writer, d.len() as u64)?;
				Varint::write_u64(&mut self.writer, d.keys().filter(|k| k.is_integer()).count() as u64)?;
				for (k, v) in d {
					match k {
						WireKey::Integer(i) => Varint::write_i64(&mut self.writer, *i)?,
						WireKey::String(s) => self.write_sized(s)?,
					}
					self.encode(v)?;
				}
				Ok(())
			}
			Wire::Id(i) => {
				self.writer.write_char('x')?;
				Varint::write_u64(&mut self.writer, i.get())
			}
			Wire::Url(u) => self.write_display('u', u.encode()),
			Wire::Bytes(b) => {
				self.writer.write_char('y')?;
				self.write_sized(b)
			}
		}
	}

	fn write_sized(&mut self, value: impl AsRef<[u8]>) -> fmt::Result {
		let value = value.as_ref();

		Varint::write_u64(&mut self.writer, Twiddle::len_bytes(value) as u64)?;
		Twiddle::encode_bytes(&mut self.writer, value)
	}

	fn write_display(&mut self, tag: char, value: impl fmt::Display) -> fmt::Result {
		let len = Twiddle::len(&value)?;

		self.writer.write_char(tag)?;
		Varint::write_u64(&mut self.writer, len as u64)?;
		Twiddle::encode(&mut self.writer, &value)
	}
}
