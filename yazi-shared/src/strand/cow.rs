use std::{borrow::Cow, ffi::{OsStr, OsString}};

use anyhow::Result;

use crate::strand::{AsStrand, Strand, StrandBuf, StrandKind};

pub enum StrandCow<'a> {
	Borrowed(Strand<'a>),
	Owned(StrandBuf),
}

impl Default for StrandCow<'_> {
	fn default() -> Self { Self::Borrowed(Strand::default()) }
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

impl<'a, T> From<&'a T> for StrandCow<'a>
where
	T: ?Sized + AsStrand,
{
	fn from(value: &'a T) -> Self { Self::Borrowed(value.as_strand()) }
}

impl PartialEq<Strand<'_>> for StrandCow<'_> {
	fn eq(&self, other: &Strand) -> bool { self.as_strand() == *other }
}

impl<'a> StrandCow<'a> {
	pub fn into_owned(self) -> StrandBuf {
		match self {
			Self::Borrowed(s) => s.to_owned(),
			Self::Owned(s) => s,
		}
	}

	pub fn into_string_lossy(self) -> String {
		match self {
			Self::Borrowed(s) => s.to_string_lossy().into_owned(),
			Self::Owned(s) => s.into_string_lossy(),
		}
	}

	pub fn with<K, T>(kind: K, bytes: T) -> Result<Self>
	where
		K: Into<StrandKind>,
		T: Into<Cow<'a, [u8]>>,
	{
		Ok(match bytes.into() {
			Cow::Borrowed(b) => Strand::with(kind, b)?.into(),
			Cow::Owned(b) => StrandBuf::with(kind, b)?.into(),
		})
	}
}
