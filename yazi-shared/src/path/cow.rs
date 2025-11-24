use std::borrow::Cow;

use anyhow::Result;

use crate::path::{AsPath, PathBufDyn, PathDyn, PathKind};

// --- PathCow
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

impl From<PathCow<'_>> for PathBufDyn {
	fn from(value: PathCow<'_>) -> Self { value.into_owned() }
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
			Self::Borrowed(s) => s.to_owned(),
			Self::Owned(s) => s,
		}
	}

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
