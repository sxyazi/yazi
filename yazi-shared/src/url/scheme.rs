use std::{borrow::Cow, fmt::Display};

use anyhow::{Result, bail};
use percent_encoding::{AsciiSet, CONTROLS, PercentEncode, percent_decode, percent_encode};

use crate::{BytesExt, url::Loc};

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
	pub(super) fn parse(bytes: &[u8]) -> Result<(Self, usize, bool)> {
		let Some((mut protocol, rest)) = bytes.split_by_seq(b"://") else {
			return Ok((Self::Regular, 0, false));
		};

		let tilde = protocol.ends_with(b"~");
		if tilde {
			protocol = &protocol[..protocol.len() - 1];
		}

		Ok(match protocol {
			b"regular" => (Self::Regular, 10, tilde),
			b"search" => {
				let (name, skip) = Self::decode_param(rest)?;
				(Self::Search(name), 9 + skip, tilde)
			}
			b"archive" => {
				let (name, skip) = Self::decode_param(rest)?;
				(Self::Archive(name), 10 + skip, tilde)
			}
			b"sftp" => {
				let (name, skip) = Self::decode_param(rest)?;
				(Self::Sftp(name), 7 + skip, tilde)
			}
			_ => bail!("Could not parse protocol from URL: {}", String::from_utf8_lossy(bytes)),
		})
	}

	#[inline]
	pub fn covariant(&self, other: &Self) -> bool {
		if self.is_virtual() || other.is_virtual() { self == other } else { true }
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
	fn encode_param<'a>(s: &'a str) -> PercentEncode<'a> {
		const SET: AsciiSet = CONTROLS.add(b'/');
		percent_encode(s.as_bytes(), &SET)
	}

	pub fn encode_tilded(&self, loc: &Loc) -> String {
		let loc = percent_encode(loc.as_os_str().as_encoded_bytes(), CONTROLS);
		match self {
			Self::Regular => format!("regular~://{loc}"),
			Self::Search(kw) => format!("search~://{}/{loc}", Self::encode_param(kw)),
			Self::SearchItem => format!("search-item~://{loc}"),
			Self::Archive(id) => format!("archive~://{}/{loc}", Self::encode_param(id)),
			Self::Sftp(id) => format!("sftp~://{}/{loc}", Self::encode_param(id)),
		}
	}
}

impl Display for Scheme {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Regular => write!(f, "regular://"),
			Self::Search(kw) => write!(f, "search://{}/", Self::encode_param(kw)),
			Self::SearchItem => write!(f, "search-item://"),
			Self::Archive(id) => write!(f, "archive://{}/", Self::encode_param(id)),
			Self::Sftp(id) => write!(f, "sftp://{}/", Self::encode_param(id)),
		}
	}
}
