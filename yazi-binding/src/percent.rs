use std::{borrow::Cow, str};

use mlua::{BorrowedBytes, FromLua, Lua, Table, Value};
use percent_encoding::{AsciiSet, CONTROLS, NON_ALPHANUMERIC, percent_encode_byte};

// RFC 3986 path component: encode everything that is not a safe path character.
// Safe chars: unreserved (A-Za-z0-9 -._~) + sub-delims (!$&'()*+,;=) + :@/
const RFC_3986: AsciiSet = CONTROLS
	.add(b' ')
	.add(b'"')
	.add(b'#')
	.add(b'%')
	.add(b'<')
	.add(b'>')
	.add(b'?')
	.add(b'[')
	.add(b'\\')
	.add(b']')
	.add(b'^')
	.add(b'`')
	.add(b'{')
	.add(b'|')
	.add(b'}');

// RFC 3986 query parameter: encode everything that is not an unreserved character.
// Safe chars: unreserved (A-Za-z0-9-._~)
const QUERY_COMPONENT: AsciiSet =
	NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

// --- PercentMode
pub struct PercentMode(AsciiSet);

impl Default for PercentMode {
	fn default() -> Self { Self(RFC_3986) }
}

impl PercentMode {
	pub fn set(&self) -> &AsciiSet { &self.0 }

	pub fn encode<'a>(&self, input: &'a [u8]) -> Cow<'a, str> {
		let set = self.set();

		let need = |b: u8| !b.is_ascii() || set.add(b) == *set;
		let Some(i) = input.iter().position(|&b| need(b)) else {
			return Cow::Borrowed(unsafe { str::from_utf8_unchecked(input) });
		};

		let mut output = String::with_capacity(input.len() + 2);
		output.push_str(unsafe { str::from_utf8_unchecked(&input[..i]) });
		for &b in &input[i..] {
			if need(b) {
				output.push_str(percent_encode_byte(b));
			} else {
				output.push(b as char);
			}
		}
		Cow::Owned(output)
	}
}

impl TryFrom<Table> for PercentMode {
	type Error = mlua::Error;

	fn try_from(table: Table) -> Result<Self, Self::Error> {
		let mode: BorrowedBytes = table.raw_get(1)?;
		let mut set = match &*mode {
			b"path" => RFC_3986,
			b"query" => QUERY_COMPONENT,
			_ => return Err(mlua::Error::runtime("Invalid percent-encode mode")),
		};

		if let Some(extra) = table.raw_get::<Option<BorrowedBytes>>("extra")? {
			for &byte in &extra {
				if !byte.is_ascii() {
					return Err(mlua::Error::runtime("Percent-encode extra must be ASCII"));
				}
				set = set.add(byte);
			}
		}
		Ok(Self(set))
	}
}

impl FromLua for PercentMode {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(Self::default()),
			Value::Table(t) => t.try_into(),
			_ => Err(mlua::Error::runtime("Expected percent-encode options")),
		}
	}
}
