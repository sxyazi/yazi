use std::{borrow::Cow, collections::BTreeMap};

use anyhow::{Result, anyhow, bail, ensure};
use yazi_shim::utf8::IntoUtf8;

use super::{WireKey, varint::Varint, wire::Wire};
use crate::{id::Id, twiddle::Twiddle, url::UrlCow};

pub(super) struct Decoder<'a> {
	bytes: &'a [u8],
	pos:   usize,
}

impl<'a> Decoder<'a> {
	pub(super) fn new(bytes: &'a [u8]) -> Self { Self { bytes, pos: 0 } }

	pub(super) fn decode(mut self) -> Result<Wire> {
		let data = self.value()?;
		ensure!(self.pos == self.bytes.len(), "trailing bytes in wire data");
		Ok(data)
	}

	fn value(&mut self) -> Result<Wire> {
		match self.byte()? {
			b'n' => Ok(Wire::Nil),
			b'0' => Ok(false.into()),
			b'1' => Ok(true.into()),
			b'i' => Ok(self.signed_varint()?.into()),
			b'f' => {
				let b = self.sized()?;
				let f: f64 = str::from_utf8(&b)?.parse()?;
				Ok(f.into())
			}
			b's' => Ok(self.sized()?.into_utf8()?.into_owned().into()),
			tag @ b'A'..=b'Z' => self.list(usize::from(tag - b'A')),
			b'l' => {
				let len = self.varint()?.try_into()?;
				self.list(len)
			}
			b'!' => Ok(BTreeMap::new().into()),
			b'd' => {
				let len = self.varint()?.try_into()?;
				let integers = self.varint()?.try_into()?;
				self.dict(len, integers)
			}
			b'x' => Ok(Id::from(self.varint()?).into()),
			b'u' => Ok(UrlCow::try_from(&*self.sized()?)?.into_owned().into()),
			b'y' => Ok(self.sized()?.into_owned().into()),
			tag => bail!("invalid wire data tag: {tag:?}"),
		}
	}

	fn byte(&mut self) -> Result<u8> {
		let byte = *self.bytes.get(self.pos).ok_or_else(|| anyhow!("unexpected end of wire data"))?;
		self.pos += 1;
		Ok(byte)
	}

	fn varint(&mut self) -> Result<u64> {
		let (value, len) = Varint::read_u64(&self.bytes[self.pos..])?;
		self.pos += len;
		Ok(value)
	}

	fn signed_varint(&mut self) -> Result<i64> {
		let (value, len) = Varint::read_i64(&self.bytes[self.pos..])?;
		self.pos += len;
		Ok(value)
	}

	fn sized(&mut self) -> Result<Cow<'a, [u8]>> {
		let len = self.varint()?.try_into()?;
		let end = self.pos.checked_add(len).ok_or_else(|| anyhow!("wire data length overflow"))?;
		let value = self.bytes.get(self.pos..end).ok_or_else(|| anyhow!("wire data is truncated"))?;
		self.pos = end;
		Twiddle::decode(value)
	}

	fn list(&mut self, len: usize) -> Result<Wire> {
		ensure!(len <= self.bytes.len().saturating_sub(self.pos), "invalid list length in wire data");

		let mut values = Vec::with_capacity(len);
		for _ in 0..len {
			values.push(self.value()?);
		}
		Ok(values.into())
	}

	fn dict(&mut self, len: usize, integers: usize) -> Result<Wire> {
		ensure!(len <= self.bytes.len().saturating_sub(self.pos), "invalid dict length in wire data");
		ensure!(integers <= len, "invalid integer key count in wire data");

		let mut values = BTreeMap::new();
		for i in 0..len {
			let key = if i < integers {
				WireKey::Integer(self.signed_varint()?)
			} else {
				WireKey::String(self.sized()?.into_utf8()?.into_owned())
			};
			let old = values.insert(key, self.value()?);
			ensure!(old.is_none(), "duplicate dict key in wire data");
		}
		Ok(values.into())
	}
}
