use std::{borrow::Cow, ffi::OsStr};

use anyhow::Result;
use hashbrown::Equivalent;

use super::{RsplitOnceError, StartsWithError};
use crate::{BytesExt, FromWtf8, Utf8BytePredictor, path::{AsPath, EndsWithError, JoinError, PathBufDyn, PathDynError, PathKind, StripPrefixError}, strand::{AsStrand, Strand, StrandError}};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathDyn<'p> {
	Os(&'p std::path::Path),
}

impl<'a> From<&'a std::path::Path> for PathDyn<'a> {
	fn from(value: &'a std::path::Path) -> Self { Self::Os(value) }
}

impl<'a> From<&'a PathBufDyn> for PathDyn<'a> {
	fn from(value: &'a PathBufDyn) -> Self { value.as_path() }
}

impl PartialEq<PathBufDyn> for PathDyn<'_> {
	fn eq(&self, other: &PathBufDyn) -> bool { *self == other.as_path() }
}

impl PartialEq<PathDyn<'_>> for &std::path::Path {
	fn eq(&self, other: &PathDyn<'_>) -> bool { matches!(*other, PathDyn::Os(p) if p == *self) }
}

impl PartialEq<&std::path::Path> for PathDyn<'_> {
	fn eq(&self, other: &&std::path::Path) -> bool { matches!(*self, PathDyn::Os(p) if p == *other) }
}

impl PartialEq<&str> for PathDyn<'_> {
	fn eq(&self, other: &&str) -> bool {
		match self {
			PathDyn::Os(p) => p == other,
		}
	}
}

impl Equivalent<PathBufDyn> for PathDyn<'_> {
	fn equivalent(&self, key: &PathBufDyn) -> bool { *self == key.as_path() }
}

impl<'p> PathDyn<'p> {
	#[inline]
	pub fn as_os(self) -> Result<&'p std::path::Path, PathDynError> {
		match self {
			Self::Os(p) => Ok(p),
		}
	}

	pub fn components(self) -> std::path::Components<'p> {
		match self {
			Self::Os(p) => p.components(),
		}
	}

	pub fn display(self) -> std::path::Display<'p> {
		match self {
			Self::Os(p) => p.display(),
		}
	}

	pub fn encoded_bytes(self) -> &'p [u8] {
		match self {
			Self::Os(p) => p.as_os_str().as_encoded_bytes(),
		}
	}

	pub fn ext(self) -> Option<Strand<'p>> {
		Some(match self {
			Self::Os(p) => p.extension()?.into(),
		})
	}

	#[inline]
	pub unsafe fn from_encoded_bytes<K>(kind: K, bytes: &'p [u8]) -> Self
	where
		K: Into<PathKind>,
	{
		match kind.into() {
			PathKind::Os => Self::Os(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) }.as_ref()),
		}
	}

	pub fn has_root(self) -> bool {
		match self {
			Self::Os(p) => p.has_root(),
		}
	}

	pub fn is_absolute(self) -> bool {
		match self {
			Self::Os(p) => p.is_absolute(),
		}
	}

	pub fn is_empty(self) -> bool { self.encoded_bytes().is_empty() }

	#[cfg(unix)]
	pub fn is_hidden(self) -> bool {
		self.name().is_some_and(|n| n.encoded_bytes().first() == Some(&b'.'))
	}

	pub fn kind(self) -> PathKind {
		match self {
			Self::Os(_) => PathKind::Os,
		}
	}

	pub fn len(self) -> usize { self.encoded_bytes().len() }

	pub fn name(self) -> Option<Strand<'p>> {
		Some(match self {
			Self::Os(p) => p.file_name()?.into(),
		})
	}

	#[inline]
	pub fn os<T>(path: &'p T) -> Self
	where
		T: ?Sized + AsRef<std::path::Path>,
	{
		Self::Os(path.as_ref())
	}

	pub fn parent(self) -> Option<Self> {
		Some(match self {
			Self::Os(p) => Self::Os(p.parent()?),
		})
	}

	pub fn rsplit_pred<T>(self, pred: T) -> Option<(Self, Self)>
	where
		T: Utf8BytePredictor,
	{
		let (a, b) = self.encoded_bytes().rsplit_pred_once(pred)?;
		Some(unsafe {
			(Self::from_encoded_bytes(self.kind(), a), Self::from_encoded_bytes(self.kind(), b))
		})
	}

	pub fn stem(self) -> Option<Strand<'p>> {
		Some(match self {
			Self::Os(p) => p.file_stem()?.into(),
		})
	}

	#[inline]
	pub fn to_os_owned(self) -> Result<std::path::PathBuf, PathDynError> {
		match self {
			Self::Os(p) => Ok(p.to_owned()),
		}
	}

	pub fn to_owned(self) -> PathBufDyn {
		match self {
			Self::Os(p) => PathBufDyn::Os(p.to_path_buf()),
		}
	}

	pub fn to_str(self) -> Result<&'p str, std::str::Utf8Error> {
		str::from_utf8(self.encoded_bytes())
	}

	pub fn to_string_lossy(self) -> Cow<'p, str> { String::from_utf8_lossy(self.encoded_bytes()) }

	pub fn try_ends_with<T>(self, child: T) -> Result<bool, EndsWithError>
	where
		T: AsStrand,
	{
		Ok(match (self, child.as_strand()) {
			(Self::Os(p), Strand::Os(q)) => p.ends_with(q),
			(Self::Os(p), Strand::Utf8(q)) => p.ends_with(q),
			(Self::Os(p), Strand::Bytes(b)) => {
				p.ends_with(OsStr::from_wtf8(b).map_err(|_| EndsWithError)?)
			}
		})
	}

	pub fn try_join<T>(self, path: T) -> Result<PathBufDyn, JoinError>
	where
		T: AsStrand,
	{
		Ok(match (self, path.as_strand()) {
			(Self::Os(p), Strand::Os(q)) => PathBufDyn::Os(p.join(q)),
			(Self::Os(p), Strand::Utf8(q)) => PathBufDyn::Os(p.join(q)),
			(Self::Os(p), Strand::Bytes(b)) => {
				PathBufDyn::Os(p.join(OsStr::from_wtf8(b).map_err(|_| JoinError::FromWtf8)?))
			}
		})
	}

	pub fn try_rsplit_seq<T>(self, pat: T) -> Result<(Self, Self), RsplitOnceError>
	where
		T: AsStrand,
	{
		let pat = pat.as_strand();

		let (a, b) = match self {
			PathDyn::Os(p) => {
				p.as_os_str().as_encoded_bytes().rsplit_seq_once(pat.as_os()?.as_encoded_bytes())
			}
		}
		.ok_or(RsplitOnceError::NotFound)?;

		Ok(unsafe {
			(Self::from_encoded_bytes(self.kind(), a), Self::from_encoded_bytes(self.kind(), b))
		})
	}

	pub fn try_starts_with<T>(self, base: T) -> Result<bool, StartsWithError>
	where
		T: AsStrand,
	{
		Ok(match (self, base.as_strand()) {
			(Self::Os(p), Strand::Os(q)) => p.starts_with(q),
			(Self::Os(p), Strand::Utf8(q)) => p.starts_with(q),
			(Self::Os(p), Strand::Bytes(b)) => {
				p.starts_with(OsStr::from_wtf8(b).map_err(|_| StartsWithError)?)
			}
		})
	}

	pub fn try_strip_prefix<T>(self, base: T) -> Result<Self, StripPrefixError>
	where
		T: AsStrand,
	{
		Ok(match (self, base.as_strand()) {
			(Self::Os(p), Strand::Os(q)) => Self::Os(p.strip_prefix(q)?),
			(Self::Os(p), Strand::Utf8(q)) => Self::Os(p.strip_prefix(q)?),
			(Self::Os(p), Strand::Bytes(b)) => {
				Self::Os(p.strip_prefix(OsStr::from_wtf8(b).map_err(|_| StripPrefixError::WrongEncoding)?)?)
			}
		})
	}

	pub fn with<K, T>(kind: K, strand: &'p T) -> Result<Self, StrandError>
	where
		K: Into<PathKind>,
		T: ?Sized + AsStrand,
	{
		let s = strand.as_strand();
		Ok(match kind.into() {
			PathKind::Os => Self::Os(std::path::Path::new(s.as_os()?)),
		})
	}
}
