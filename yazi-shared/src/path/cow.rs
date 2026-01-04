use std::borrow::Cow;

use anyhow::Result;

use crate::path::{AsPath, PathBufDyn, PathDyn, PathDynError, PathKind};

// --- PathCow
#[derive(Debug)]
pub enum PathCow<'a> {
	Borrowed(PathDyn<'a>),
	Owned(PathBufDyn),
}

impl<'a> From<PathDyn<'a>> for PathCow<'a> {
	fn from(value: PathDyn<'a>) -> Self { Self::Borrowed(value) }
}

impl From<PathBufDyn> for PathCow<'_> {
	fn from(value: PathBufDyn) -> Self { Self::Owned(value) }
}

impl<'a> From<std::path::PathBuf> for PathCow<'a> {
	fn from(value: std::path::PathBuf) -> Self { Self::Owned(value.into()) }
}

impl<'a> From<&'a PathCow<'_>> for PathCow<'a> {
	fn from(value: &'a PathCow<'_>) -> Self { Self::Borrowed(value.as_path()) }
}

impl From<PathCow<'_>> for PathBufDyn {
	fn from(value: PathCow<'_>) -> Self { value.into_owned() }
}

impl PartialEq for PathCow<'_> {
	fn eq(&self, other: &Self) -> bool { self.as_path() == other.as_path() }
}

impl PartialEq<&str> for PathCow<'_> {
	fn eq(&self, other: &&str) -> bool {
		match self {
			Self::Borrowed(s) => s.as_path() == *other,
			Self::Owned(s) => s.as_path() == *other,
		}
	}
}

impl<'a> PathCow<'a> {
	pub fn into_owned(self) -> PathBufDyn {
		match self {
			Self::Borrowed(p) => p.to_owned(),
			Self::Owned(p) => p,
		}
	}

	pub fn into_os(self) -> Result<std::path::PathBuf, PathDynError> {
		match self {
			PathCow::Borrowed(p) => p.to_os_owned(),
			PathCow::Owned(p) => p.into_os(),
		}
	}

	pub fn is_borrowed(&self) -> bool { matches!(self, Self::Borrowed(_)) }

	pub fn with<K, T>(kind: K, bytes: T) -> Result<Self>
	where
		K: Into<PathKind>,
		T: Into<Cow<'a, [u8]>>,
	{
		Ok(match bytes.into() {
			Cow::Borrowed(b) => PathDyn::with(kind, b)?.into(),
			Cow::Owned(b) => PathBufDyn::with(kind, b)?.into(),
		})
	}
}
