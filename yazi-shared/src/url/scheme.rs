use std::{borrow::Cow, fmt::Display};

use anyhow::{Result, bail};
use percent_encoding::{CONTROLS, PercentEncode, percent_decode, percent_encode};

use crate::BytesExt;

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Scheme {
	#[default]
	Regular,

	Search(String),
	SearchItem,

	Archive(String),

	Sftp(String),
}

impl Scheme {
	pub(super) fn parse(bytes: &[u8]) -> Result<(Self, usize)> {
		let Some((protocol, rest)) = bytes.split_by_seq(b"://") else {
			return Ok((Self::Regular, 0));
		};

		Ok(match protocol {
			b"regular" => (Self::Regular, 10),
			b"search" => {
				let (name, skip) = Self::decode_param(rest)?;
				(Self::Search(name), 9 + skip)
			}
			b"archive" => {
				let (name, skip) = Self::decode_param(rest)?;
				(Self::Archive(name), 10 + skip)
			}
			b"sftp" => {
				let (name, skip) = Self::decode_param(rest)?;
				(Self::Sftp(name), 7 + skip)
			}
			_ => bail!("Could not parse protocol from URL: {}", String::from_utf8_lossy(bytes)),
		})
	}

	pub fn covariant(&self, other: &Self) -> bool {
		match (self, other) {
			// Local files
			(
				Self::Regular | Self::Search(_) | Self::SearchItem,
				Self::Regular | Self::Search(_) | Self::SearchItem,
			) => true,

			// Virtual files within the same namespace
			(a, b) => a == b,
		}
	}

	#[inline]
	pub(super) fn is_virtual(&self) -> bool {
		match self {
			Self::Regular | Self::Search(_) | Self::SearchItem => false,
			Self::Archive(_) | Self::Sftp(_) => true,
		}
	}

	fn decode_param(bytes: &[u8]) -> Result<(String, usize)> {
		let len = bytes.iter().copied().take_while(|&b| b != b'/').count();

		let s = match Cow::from(percent_decode(&bytes[..len])) {
			Cow::Borrowed(b) => str::from_utf8(b)?.to_owned(),
			Cow::Owned(b) => String::from_utf8(b)?,
		};

		let slash = bytes.get(len).is_some_and(|&b| b == b'/') as usize;
		Ok((s, len + slash))
	}

	#[inline]
	fn encode_param<'a>(s: &'a str) -> PercentEncode<'a> { percent_encode(s.as_bytes(), CONTROLS) }
}

impl Display for Scheme {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Regular => write!(f, "regular://"),
			Self::Search(kw) => write!(f, "search://{}/", Self::encode_param(kw)),
			Self::SearchItem => write!(f, "search_item://"),
			Self::Archive(id) => write!(f, "archive://{}/", Self::encode_param(id)),
			Self::Sftp(id) => write!(f, "sftp://{}/", Self::encode_param(id)),
		}
	}
}
