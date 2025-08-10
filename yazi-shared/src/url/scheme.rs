use std::borrow::Cow;

use anyhow::{Result, bail};
use percent_encoding::percent_decode;

use crate::BytesExt;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Scheme {
	#[default]
	Regular,

	Search(String),

	Archive(String),

	Sftp(String),
}

impl Scheme {
	#[inline]
	pub const fn kind(&self) -> &'static str {
		match self {
			Self::Regular => "regular",
			Self::Search(_) => "search",
			Self::Archive(_) => "archive",
			Self::Sftp(_) => "sftp",
		}
	}

	#[inline]
	pub fn domain(&self) -> Option<&str> {
		match self {
			Self::Regular => None,
			Self::Search(s) | Self::Archive(s) | Self::Sftp(s) => Some(s),
		}
	}

	pub(super) fn parse(
		bytes: &[u8],
		skip: &mut usize,
	) -> Result<(Self, bool, Option<(usize, usize)>)> {
		let Some((mut protocol, rest)) = bytes.split_by_seq(b"://") else {
			return Ok((Self::Regular, false, None));
		};

		// Advance to the beginning of the path
		*skip += 3 + protocol.len();

		// Tilded schemes
		let tilde = protocol.ends_with(b"~");
		if tilde {
			protocol = &protocol[..protocol.len() - 1];
		}

		let (scheme, port) = match protocol {
			b"regular" => (Self::Regular, None),
			b"search" => {
				let (domain, uri, urn) = Self::decode_param(rest, skip)?;
				(Self::Search(domain), Some((uri, urn)))
			}
			b"archive" => {
				let (domain, uri, urn) = Self::decode_param(rest, skip)?;
				(Self::Archive(domain), Some((uri, urn)))
			}
			b"sftp" => {
				let (domain, uri, urn) = Self::decode_param(rest, skip)?;
				(Self::Sftp(domain), Some((uri, urn)))
			}
			_ => bail!("Could not parse protocol from URL: {}", String::from_utf8_lossy(bytes)),
		};

		Ok((scheme, tilde, port))
	}

	#[inline]
	pub fn parse_kind(bytes: &[u8]) -> Result<&'static str> {
		match bytes {
			b"regular" => Ok("regular"),
			b"search" => Ok("search"),
			b"archive" => Ok("archive"),
			b"sftp" => Ok("sftp"),
			_ => bail!("Could not parse protocol from URL: {}", String::from_utf8_lossy(bytes)),
		}
	}

	#[inline]
	pub fn covariant(&self, other: &Self) -> bool {
		if self.is_virtual() || other.is_virtual() { self == other } else { true }
	}

	#[inline]
	pub fn is_virtual(&self) -> bool {
		match self {
			Self::Regular | Self::Search(_) => false,
			Self::Archive(_) | Self::Sftp(_) => true,
		}
	}

	fn decode_param(bytes: &[u8], skip: &mut usize) -> Result<(String, usize, usize)> {
		let mut len = bytes.iter().copied().take_while(|&b| b != b'/').count();
		let slash = bytes.get(len).is_some_and(|&b| b == b'/');
		*skip += len + slash as usize;

		let (uri, urn) = Self::decode_port(&bytes[..len], &mut len)?;
		let domain = match Cow::from(percent_decode(&bytes[..len])) {
			Cow::Borrowed(b) => str::from_utf8(b)?.to_owned(),
			Cow::Owned(b) => String::from_utf8(b)?,
		};

		Ok((domain, uri, urn))
	}

	fn decode_port(bytes: &[u8], skip: &mut usize) -> anyhow::Result<(usize, usize)> {
		let Some(a_idx) = bytes.iter().rposition(|&b| b == b':') else { return Ok((0, 0)) };
		let a_len = bytes.len() - a_idx;
		*skip -= a_len;
		let a = if a_len == 1 { 0 } else { str::from_utf8(&bytes[a_idx + 1..])?.parse()? };

		let Some(b_idx) = bytes[..a_idx].iter().rposition(|&b| b == b':') else { return Ok((a, 0)) };
		let b_len = bytes[..a_idx].len() - b_idx;
		*skip -= b_len;
		let b = if b_len == 1 { 0 } else { str::from_utf8(&bytes[b_idx + 1..a_idx])?.parse()? };

		Ok((b, a))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_decode_port() -> Result<()> {
		fn assert(s: &str, uri: usize, urn: usize, len: usize) -> Result<()> {
			let mut n = usize::MAX;
			let port = Scheme::decode_port(s.as_bytes(), &mut n)?;
			assert_eq!((port.0, port.1, usize::MAX - n), (uri, urn, len));
			Ok(())
		}

		// Zeros
		assert("", 0, 0, 0)?;
		assert(":", 0, 0, 1)?;
		assert("::", 0, 0, 2)?;

		// URI
		assert(":2", 2, 0, 2)?;
		assert(":2:", 2, 0, 3)?;
		assert(":22:", 22, 0, 4)?;

		// URN
		assert("::1", 0, 1, 3)?;
		assert(":2:1", 2, 1, 4)?;
		assert(":22:11", 22, 11, 6)?;
		Ok(())
	}
}
