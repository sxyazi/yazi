use anyhow::{Result, bail};

use crate::{BytesExt, scheme::{AsScheme, SchemeRef}};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SchemeKind {
	Regular,
	Search,
	Archive,
	Sftp,
}

impl<T> From<T> for SchemeKind
where
	T: AsScheme,
{
	fn from(value: T) -> Self {
		match value.as_scheme() {
			SchemeRef::Regular { .. } => Self::Regular,
			SchemeRef::Search { .. } => Self::Search,
			SchemeRef::Archive { .. } => Self::Archive,
			SchemeRef::Sftp { .. } => Self::Sftp,
		}
	}
}

impl TryFrom<&[u8]> for SchemeKind {
	type Error = anyhow::Error;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		match value {
			b"regular" => Ok(Self::Regular),
			b"search" => Ok(Self::Search),
			b"archive" => Ok(Self::Archive),
			b"sftp" => Ok(Self::Sftp),
			_ => bail!("invalid scheme kind: {}", String::from_utf8_lossy(value)),
		}
	}
}

impl SchemeKind {
	#[inline]
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::Regular => "regular",
			Self::Search => "search",
			Self::Archive => "archive",
			Self::Sftp => "sftp",
		}
	}

	#[inline]
	pub fn is_local(self) -> bool {
		match self {
			Self::Regular | Self::Search => true,
			Self::Archive | Self::Sftp => false,
		}
	}

	#[inline]
	pub fn is_remote(self) -> bool {
		match self {
			Self::Regular | Self::Search | Self::Archive => false,
			Self::Sftp => true,
		}
	}

	#[inline]
	pub fn is_virtual(self) -> bool {
		match self {
			Self::Regular | Self::Search => false,
			Self::Archive | Self::Sftp => true,
		}
	}

	#[inline]
	pub(super) const fn offset(self, tilde: bool) -> usize {
		3 + self.as_str().len() + tilde as usize
	}

	pub fn parse(bytes: &[u8]) -> Result<Option<(Self, bool)>> {
		let Some((kind, _)) = bytes.split_seq_once(b"://") else {
			return Ok(None);
		};

		Ok(Some(if let Some(stripped) = kind.strip_suffix(b"~") {
			(Self::try_from(stripped)?, true)
		} else {
			(Self::try_from(kind)?, false)
		}))
	}
}
