use std::{borrow::Cow, ffi::{OsStr, OsString}};

use anyhow::Result;

use crate::{IntoOsStr, scheme::SchemeKind, strand::{Strand, StrandBuf}};

// --- StrandCow
pub enum StrandCow<'a> {
	Borrowed(Strand<'a>),
	Owned(StrandBuf),
}

impl From<OsString> for StrandCow<'_> {
	fn from(value: OsString) -> Self { Self::Owned(StrandBuf::Os(value)) }
}

impl<'a> From<Cow<'a, OsStr>> for StrandCow<'a> {
	fn from(value: Cow<'a, OsStr>) -> Self {
		match value {
			Cow::Borrowed(s) => Self::Borrowed(Strand::Os(s)),
			Cow::Owned(s) => Self::Owned(StrandBuf::Os(s)),
		}
	}
}

impl<'a> From<Strand<'a>> for StrandCow<'a> {
	fn from(value: Strand<'a>) -> Self { Self::Borrowed(value) }
}

impl From<StrandBuf> for StrandCow<'_> {
	fn from(value: StrandBuf) -> Self { Self::Owned(value) }
}

impl PartialEq<Strand<'_>> for StrandCow<'_> {
	fn eq(&self, other: &Strand) -> bool {
		match self {
			Self::Borrowed(s) => s == other,
			Self::Owned(s) => s == other,
		}
	}
}

impl<'a> StrandCow<'a> {
	pub fn from_os_bytes(bytes: impl Into<Cow<'a, [u8]>>) -> Result<Self> {
		Ok(match bytes.into().into_os_str()? {
			Cow::Borrowed(s) => Strand::Os(s).into(),
			Cow::Owned(s) => StrandBuf::Os(s).into(),
		})
	}

	pub fn into_owned(self) -> StrandBuf {
		match self {
			Self::Borrowed(s) => s.to_owned(),
			Self::Owned(s) => s,
		}
	}

	pub fn with<T>(kind: SchemeKind, bytes: T) -> Result<Self>
	where
		T: Into<Cow<'a, [u8]>>,
	{
		match kind {
			SchemeKind::Regular | SchemeKind::Search | SchemeKind::Archive => Self::from_os_bytes(bytes),
			SchemeKind::Sftp => Self::from_os_bytes(bytes), // FIXME
		}
	}
}
