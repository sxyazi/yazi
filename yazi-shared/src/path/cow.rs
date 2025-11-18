use std::borrow::Cow;

use anyhow::Result;

use crate::{IntoOsStr, path::{AsPathDyn, PathBufDyn, PathBufLike, PathDyn, PathLike}};

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
			Self::Borrowed(s) => s.as_path_dyn() == *other,
			Self::Owned(s) => s.as_path_dyn() == *other,
		}
	}
}

impl<'a> PathCow<'a> {
	pub fn from_os_bytes(bytes: impl Into<Cow<'a, [u8]>>) -> Result<Self> {
		Ok(match bytes.into().into_os_str()? {
			Cow::Borrowed(s) => PathDyn::os(s).into(),
			Cow::Owned(s) => PathBufDyn::os(s).into(),
		})
	}

	pub fn into_owned(self) -> PathBufDyn {
		match self {
			Self::Borrowed(s) => s.to_buf_dyn(),
			Self::Owned(s) => s,
		}
	}

	// FIXME: remove, instead implement PathLike for PathCow
	pub fn encoded_bytes(&self) -> &[u8] {
		match self {
			Self::Borrowed(s) => s.encoded_bytes(),
			Self::Owned(s) => s.encoded_bytes(),
		}
	}
}
