use std::{borrow::Cow, ffi::OsStr};

use anyhow::Result;
use hashbrown::Equivalent;

use super::{RsplitOnceError, StartsWithError};
use crate::{BytesExt, Utf8BytePredictor, path::{AsPath, Components, Display, EndsWithError, JoinError, PathBufDyn, PathDynError, PathKind, StripPrefixError, StripSuffixError}, strand::{AsStrand, Strand, StrandError}};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PathDyn<'p> {
	Os(&'p std::path::Path),
	Unix(&'p typed_path::UnixPath),
}

impl<'a> From<&'a std::path::Path> for PathDyn<'a> {
	fn from(value: &'a std::path::Path) -> Self { Self::Os(value) }
}

impl<'a> From<&'a typed_path::UnixPath> for PathDyn<'a> {
	fn from(value: &'a typed_path::UnixPath) -> Self { Self::Unix(value) }
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
		match *self {
			PathDyn::Os(p) => p == *other,
			PathDyn::Unix(p) => p == typed_path::UnixPath::new(other),
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
			Self::Unix(_) => Err(PathDynError::AsOs),
		}
	}

	#[inline]
	pub fn as_unix(self) -> Result<&'p typed_path::UnixPath, PathDynError> {
		match self {
			Self::Os(_) => Err(PathDynError::AsUnix),
			Self::Unix(p) => Ok(p),
		}
	}

	pub fn components(self) -> Components<'p> {
		match self {
			Self::Os(p) => Components::Os(p.components()),
			Self::Unix(p) => Components::Unix(p.components()),
		}
	}

	pub fn display(self) -> Display<'p> { Display(self) }

	pub fn encoded_bytes(self) -> &'p [u8] {
		match self {
			Self::Os(p) => p.as_os_str().as_encoded_bytes(),
			Self::Unix(p) => p.as_bytes(),
		}
	}

	pub fn ext(self) -> Option<Strand<'p>> {
		Some(match self {
			Self::Os(p) => p.extension()?.into(),
			Self::Unix(p) => p.extension()?.into(),
		})
	}

	#[inline]
	pub unsafe fn from_encoded_bytes<K>(kind: K, bytes: &'p [u8]) -> Self
	where
		K: Into<PathKind>,
	{
		match kind.into() {
			PathKind::Os => Self::Os(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) }.as_ref()),
			PathKind::Unix => Self::Unix(typed_path::UnixPath::new(bytes)),
		}
	}

	pub fn has_root(self) -> bool {
		match self {
			Self::Os(p) => p.has_root(),
			Self::Unix(p) => p.has_root(),
		}
	}

	pub fn is_absolute(self) -> bool {
		match self {
			Self::Os(p) => p.is_absolute(),
			Self::Unix(p) => p.is_absolute(),
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
			Self::Unix(_) => PathKind::Unix,
		}
	}

	pub fn len(self) -> usize { self.encoded_bytes().len() }

	pub fn name(self) -> Option<Strand<'p>> {
		Some(match self {
			Self::Os(p) => p.file_name()?.into(),
			Self::Unix(p) => p.file_name()?.into(),
		})
	}

	pub fn parent(self) -> Option<Self> {
		Some(match self {
			Self::Os(p) => Self::Os(p.parent().filter(|p| !p.as_os_str().is_empty())?),
			Self::Unix(p) => Self::Unix(p.parent().filter(|p| !p.as_bytes().is_empty())?),
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
			Self::Unix(p) => p.file_stem()?.into(),
		})
	}

	#[inline]
	pub fn to_os_owned(self) -> Result<std::path::PathBuf, PathDynError> {
		match self {
			Self::Os(p) => Ok(p.to_owned()),
			Self::Unix(_) => Err(PathDynError::AsOs),
		}
	}

	pub fn to_owned(self) -> PathBufDyn {
		match self {
			Self::Os(p) => PathBufDyn::Os(p.to_owned()),
			Self::Unix(p) => PathBufDyn::Unix(p.to_owned()),
		}
	}

	pub fn to_str(self) -> Result<&'p str, std::str::Utf8Error> {
		str::from_utf8(self.encoded_bytes())
	}

	pub fn to_string_lossy(self) -> Cow<'p, str> { String::from_utf8_lossy(self.encoded_bytes()) }

	pub fn to_unix_owned(self) -> Result<typed_path::UnixPathBuf, PathDynError> {
		match self {
			Self::Os(_) => Err(PathDynError::AsUnix),
			Self::Unix(p) => Ok(p.to_owned()),
		}
	}

	pub fn try_ends_with<T>(self, child: T) -> Result<bool, EndsWithError>
	where
		T: AsStrand,
	{
		let s = child.as_strand();
		Ok(match self {
			Self::Os(p) => p.ends_with(s.as_os()?),
			Self::Unix(p) => p.ends_with(s.encoded_bytes()),
		})
	}

	pub fn try_join<T>(self, path: T) -> Result<PathBufDyn, JoinError>
	where
		T: AsStrand,
	{
		let s = path.as_strand();
		Ok(match self {
			Self::Os(p) => PathBufDyn::Os(p.join(s.as_os()?)),
			Self::Unix(p) => PathBufDyn::Unix(p.join(s.encoded_bytes())),
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
			PathDyn::Unix(p) => p.as_bytes().rsplit_seq_once(pat.encoded_bytes()),
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
		let s = base.as_strand();
		Ok(match self {
			Self::Os(p) => p.starts_with(s.as_os()?),
			Self::Unix(p) => p.starts_with(s.encoded_bytes()),
		})
	}

	pub fn try_strip_prefix<T>(self, base: T) -> Result<Self, StripPrefixError>
	where
		T: AsStrand,
	{
		let s = base.as_strand();
		Ok(match self {
			Self::Os(p) => Self::Os(p.strip_prefix(s.as_os()?)?),
			Self::Unix(p) => Self::Unix(p.strip_prefix(s.encoded_bytes())?),
		})
	}

	pub fn try_strip_suffix<T>(self, suffix: T) -> Result<Self, StripSuffixError>
	where
		T: AsStrand,
	{
		let s = suffix.as_strand();
		let mut me_comps = self.components();
		let mut suf_comps = match self.kind() {
			PathKind::Os => Components::Os(std::path::Path::new(s.as_os()?).components()),
			PathKind::Unix => Components::Unix(typed_path::UnixPath::new(s.encoded_bytes()).components()),
		};

		while let Some(next) = suf_comps.next_back() {
			if me_comps.next_back() != Some(next) {
				return Err(StripSuffixError::NotSuffix);
			}
		}

		Ok(me_comps.path())
	}

	pub fn with<K, S>(kind: K, strand: &'p S) -> Result<Self, StrandError>
	where
		K: Into<PathKind>,
		S: ?Sized + AsStrand,
	{
		let s = strand.as_strand();
		Ok(match kind.into() {
			PathKind::Os => Self::Os(std::path::Path::new(s.as_os()?)),
			PathKind::Unix => Self::Unix(typed_path::UnixPath::new(s.encoded_bytes())),
		})
	}

	pub fn with_str<K, S>(kind: K, s: &'p S) -> Self
	where
		K: Into<PathKind>,
		S: ?Sized + AsRef<str>,
	{
		let s = s.as_ref();
		match kind.into() {
			PathKind::Os => Self::Os(s.as_ref()),
			PathKind::Unix => Self::Unix(s.as_ref()),
		}
	}
}
