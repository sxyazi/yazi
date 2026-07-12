use std::fmt;

use anyhow::Result;

use crate::{BytesExt, auth::Scheme};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedSpec<'a> {
	bytes:      &'a [u8],
	skip:       usize,
	pub scheme: Scheme,
	pub tilde:  bool,
}

impl fmt::Display for ParsedSpec<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}{}", self.scheme, if self.tilde { "~" } else { "" })
	}
}

impl<'a> ParsedSpec<'a> {
	pub fn parse(bytes: &'a [u8]) -> Result<Self> {
		let Some((scheme, _)) = bytes.split_seq_once(b"://") else {
			return Ok(Self { bytes, skip: 0, scheme: Scheme::Regular, tilde: false });
		};

		let (scheme, tilde) = if let Some(stripped) = scheme.strip_suffix(b"~") {
			(stripped, true)
		} else {
			(scheme, false)
		};

		let scheme: Scheme = str::from_utf8(scheme)?.parse()?;
		let skip = 3 + scheme.as_str().len() + tilde as usize;
		Ok(Self { bytes, skip, scheme, tilde })
	}

	#[inline]
	pub fn has_scheme(&self) -> bool { self.skip > 0 }

	#[inline]
	pub fn rest(&self) -> &'a [u8] { &self.bytes[self.skip..] }
}
