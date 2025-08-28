use std::{borrow::Cow, ops::Not, path::Path};

use anyhow::{Result, bail, ensure};
use percent_encoding::percent_decode;

use crate::{BytesExt, pool::InternStr, scheme::{Scheme, SchemeRef}};

#[derive(Debug)]
pub enum SchemeCow<'a> {
	Borrowed(SchemeRef<'a>),
	Owned(Scheme),
}

impl Default for SchemeCow<'_> {
	fn default() -> Self { Self::Borrowed(SchemeRef::Regular) }
}

impl From<Scheme> for SchemeCow<'_> {
	fn from(value: Scheme) -> Self { Self::Owned(value) }
}

impl<'a> From<&'a Scheme> for SchemeCow<'a> {
	fn from(value: &'a Scheme) -> Self { Self::Borrowed(value.as_ref()) }
}

impl<'a> From<SchemeRef<'a>> for SchemeCow<'a> {
	fn from(value: SchemeRef<'a>) -> Self { Self::Borrowed(value) }
}

impl From<SchemeCow<'_>> for Scheme {
	fn from(value: SchemeCow<'_>) -> Self {
		match value {
			SchemeCow::Borrowed(s) => s.into(),
			SchemeCow::Owned(s) => s,
		}
	}
}

impl<'a> SchemeCow<'a> {
	pub fn search(domain: impl Into<Cow<'a, str>>) -> Self {
		match domain.into() {
			Cow::Borrowed(s) => SchemeRef::Search(s).into(),
			Cow::Owned(s) => Scheme::Search(s.intern()).into(),
		}
	}

	pub fn archive(domain: impl Into<Cow<'a, str>>) -> Self {
		match domain.into() {
			Cow::Borrowed(s) => SchemeRef::Archive(s).into(),
			Cow::Owned(s) => Scheme::Archive(s.intern()).into(),
		}
	}

	pub fn sftp(domain: impl Into<Cow<'a, str>>) -> Self {
		match domain.into() {
			Cow::Borrowed(s) => SchemeRef::Sftp(s).into(),
			Cow::Owned(s) => Scheme::Sftp(s.intern()).into(),
		}
	}

	#[inline]
	pub fn as_ref(&self) -> SchemeRef<'_> {
		match self {
			Self::Borrowed(s) => *s,
			Self::Owned(s) => s.into(),
		}
	}

	pub(crate) fn parse(
		bytes: &'a [u8],
		skip: &mut usize,
	) -> Result<(Self, bool, Option<usize>, Option<usize>)> {
		let Some((mut protocol, rest)) = bytes.split_by_seq(b"://") else {
			return Ok((Self::default(), false, None, None));
		};

		// Advance to the beginning of the path
		*skip += 3 + protocol.len();

		// Tilded schemes
		let tilde = protocol.ends_with(b"~");
		if tilde {
			protocol = &protocol[..protocol.len() - 1];
		}

		let (scheme, uri, urn) = match protocol {
			b"regular" => (Self::default(), None, None),
			b"search" => {
				let (domain, uri, urn) = Self::decode_param(rest, skip)?;
				(Self::search(domain), uri, urn)
			}
			b"archive" => {
				let (domain, uri, urn) = Self::decode_param(rest, skip)?;
				(Self::archive(domain), uri, urn)
			}
			b"sftp" => {
				let (domain, uri, urn) = Self::decode_param(rest, skip)?;
				(Self::sftp(domain), uri, urn)
			}
			_ => bail!("Could not parse protocol from URL: {}", String::from_utf8_lossy(bytes)),
		};

		Ok((scheme, tilde, uri, urn))
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

	fn decode_param(
		bytes: &'a [u8],
		skip: &mut usize,
	) -> Result<(Cow<'a, str>, Option<usize>, Option<usize>)> {
		let mut len = bytes.iter().copied().take_while(|&b| b != b'/').count();
		let slash = bytes.get(len).is_some_and(|&b| b == b'/');
		*skip += len + slash as usize;

		let (uri, urn) = Self::decode_ports(&bytes[..len], &mut len)?;
		let domain = match Cow::from(percent_decode(&bytes[..len])) {
			Cow::Borrowed(b) => str::from_utf8(b)?.into(),
			Cow::Owned(b) => String::from_utf8(b)?.into(),
		};

		Ok((domain, uri, urn))
	}

	fn decode_ports(bytes: &[u8], skip: &mut usize) -> Result<(Option<usize>, Option<usize>)> {
		let Some(a_idx) = bytes.iter().rposition(|&b| b == b':') else { return Ok((None, None)) };
		let a_len = bytes.len() - a_idx;
		*skip -= a_len;
		let a = if a_len == 1 { None } else { Some(str::from_utf8(&bytes[a_idx + 1..])?.parse()?) };

		let Some(b_idx) = bytes[..a_idx].iter().rposition(|&b| b == b':') else {
			return Ok((a, None));
		};
		let b_len = bytes[..a_idx].len() - b_idx;
		*skip -= b_len;
		let b =
			if b_len == 1 { None } else { Some(str::from_utf8(&bytes[b_idx + 1..a_idx])?.parse()?) };

		Ok((b, a))
	}

	pub(crate) fn normalize_ports(
		&self,
		uri: Option<usize>,
		urn: Option<usize>,
		path: &Path,
	) -> Result<Option<(usize, usize)>> {
		Ok(match self.as_ref() {
			SchemeRef::Regular => {
				ensure!(uri.is_none() && urn.is_none(), "Regular scheme cannot have ports");
				None
			}
			SchemeRef::Search(_) => {
				let (uri, urn) = (uri.unwrap_or(0), urn.unwrap_or(0));
				ensure!(uri == urn, "Search scheme requires URI and URN to be equal");
				Some((uri, urn))
			}
			SchemeRef::Archive(_) => Some((uri.unwrap_or(0), urn.unwrap_or(0))),
			SchemeRef::Sftp(_) => {
				let uri = uri.unwrap_or(path.as_os_str().is_empty().not() as usize);
				let urn = urn.unwrap_or(path.file_name().is_some() as usize);
				Some((uri, urn))
			}
		})
	}
}

impl SchemeCow<'_> {
	#[inline]
	pub fn is_virtual(&self) -> bool { self.as_ref().is_virtual() }

	#[inline]
	pub fn into_owned(self) -> Scheme { self.into() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_decode_ports() -> Result<()> {
		fn assert(s: &str, len: usize, uri: Option<usize>, urn: Option<usize>) -> Result<()> {
			let mut n = usize::MAX;
			let port = SchemeCow::decode_ports(s.as_bytes(), &mut n)?;
			assert_eq!((usize::MAX - n, port.0, port.1), (len, uri, urn));
			Ok(())
		}

		// Zeros
		assert("", 0, None, None)?;
		assert(":", 1, None, None)?;
		assert("::", 2, None, None)?;

		// URI
		assert(":2", 2, Some(2), None)?;
		assert(":2:", 3, Some(2), None)?;
		assert(":22:", 4, Some(22), None)?;

		// URN
		assert("::1", 3, None, Some(1))?;
		assert(":2:1", 4, Some(2), Some(1))?;
		assert(":22:11", 6, Some(22), Some(11))?;
		Ok(())
	}
}
