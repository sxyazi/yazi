use std::ffi::OsStr;

use anyhow::Result;
use hashbrown::Equivalent;
use serde::Serialize;

use super::{AsPathView, RsplitOnceError, StartsWithError};
use crate::{FromWtf8, Utf8BytePredictor, path::{AsPathDyn, EndsWithError, JoinError, PathBufDynError, PathBufLike, PathBufUnsafeExt, PathDynError, PathKind, PathLike, SetNameError, StripPrefixError}, scheme::SchemeKind, strand::{AsStrandDyn, AsStrandView, Strand, StrandError}};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathDyn<'p> {
	Os(&'p std::path::Path),
}

impl<'a> AsPathView<'a, PathDyn<'a>> for PathDyn<'a> {
	fn as_path_view(self) -> PathDyn<'a> { self }
}

impl<'a> AsPathView<'a, PathDyn<'a>> for std::path::Components<'a> {
	fn as_path_view(self) -> PathDyn<'a> { PathDyn::Os(self.as_path()) }
}

impl<'a> From<&'a std::path::Path> for PathDyn<'a> {
	fn from(value: &'a std::path::Path) -> Self { Self::Os(value) }
}

impl<'a> From<&'a PathBufDyn> for PathDyn<'a> {
	fn from(value: &'a PathBufDyn) -> Self { value.borrow() }
}

impl PartialEq<PathBufDyn> for PathDyn<'_> {
	fn eq(&self, other: &PathBufDyn) -> bool { *self == other.borrow() }
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
	fn equivalent(&self, key: &PathBufDyn) -> bool { *self == key.borrow() }
}

impl<'p> PathLike<'p> for PathDyn<'p> {
	type Components<'a> = std::path::Components<'a>;
	type Display<'a> = std::path::Display<'a>;
	type Owned = PathBufDyn;
	type Strand<'a> = Strand<'a>;
	type View<'a> = PathDyn<'a>;

	fn as_dyn(self) -> PathDyn<'p> { self }

	fn components(self) -> Self::Components<'p> {
		match self {
			Self::Os(p) => p.components(),
		}
	}

	// FIXME: remove
	fn default() -> Self { Self::Os(std::path::Path::new("")) }

	fn display(self) -> Self::Display<'p> {
		match self {
			Self::Os(p) => p.display(),
		}
	}

	fn encoded_bytes(self) -> &'p [u8] {
		match self {
			Self::Os(p) => p.as_os_str().as_encoded_bytes(),
		}
	}

	fn ext(self) -> Option<Self::Strand<'p>> {
		Some(match self {
			Self::Os(p) => p.extension()?.into(),
		})
	}

	fn has_root(self) -> bool {
		match self {
			Self::Os(p) => p.has_root(),
		}
	}

	fn is_absolute(self) -> bool {
		match self {
			Self::Os(p) => p.is_absolute(),
		}
	}

	fn kind(self) -> PathKind {
		match self {
			Self::Os(_) => PathKind::Os,
		}
	}

	fn name(self) -> Option<Self::Strand<'p>> {
		Some(match self {
			Self::Os(p) => p.file_name()?.into(),
		})
	}

	fn owned(self) -> Self::Owned {
		match self {
			Self::Os(p) => Self::Owned::Os(p.to_path_buf()),
		}
	}

	fn parent(self) -> Option<Self> {
		Some(match self {
			Self::Os(p) => Self::Os(p.parent()?),
		})
	}

	fn stem(self) -> Option<Self::Strand<'p>> {
		Some(match self {
			Self::Os(p) => p.file_stem()?.into(),
		})
	}

	fn try_ends_with<'a, T>(self, child: T) -> Result<bool, EndsWithError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(match (self, child.as_strand_view()) {
			(Self::Os(p), Strand::Os(q)) => p.ends_with(q),
			(Self::Os(p), Strand::Utf8(q)) => p.ends_with(q),
			(Self::Os(p), Strand::Bytes(b)) => {
				p.ends_with(OsStr::from_wtf8(b).map_err(|_| EndsWithError)?)
			}
		})
	}

	fn try_join<'a, T>(self, path: T) -> Result<Self::Owned, JoinError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(match (self, path.as_strand_view()) {
			(Self::Os(p), Strand::Os(q)) => Self::Owned::Os(p.join(q)),
			(Self::Os(p), Strand::Utf8(q)) => Self::Owned::Os(p.join(q)),
			(Self::Os(p), Strand::Bytes(b)) => {
				Self::Owned::Os(p.join(OsStr::from_wtf8(b).map_err(|_| JoinError::FromWtf8)?))
			}
		})
	}

	fn rsplit_pred<'a, T>(self, pred: T) -> Option<(Self, Self)>
	where
		T: Utf8BytePredictor,
	{
		match self {
			PathDyn::Os(p) => p.rsplit_pred(pred).map(|(l, r)| (Self::Os(l), Self::Os(r))),
		}
	}

	fn try_rsplit_seq<'a, T>(self, pat: T) -> Result<(Self, Self), RsplitOnceError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		let pat = pat.as_strand_view();
		match self {
			PathDyn::Os(p) => {
				let (l, r) = p.try_rsplit_seq(pat.as_os().map_err(|_| RsplitOnceError::AsOs)?)?;
				Ok((Self::Os(l), Self::Os(r)))
			}
		}
	}

	fn try_starts_with<'a, T>(self, base: T) -> Result<bool, StartsWithError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(match (self, base.as_strand_view()) {
			(Self::Os(p), Strand::Os(q)) => p.starts_with(q),
			(Self::Os(p), Strand::Utf8(q)) => p.starts_with(q),
			(Self::Os(p), Strand::Bytes(b)) => {
				p.starts_with(OsStr::from_wtf8(b).map_err(|_| StartsWithError)?)
			}
		})
	}

	fn try_strip_prefix<'a, T>(self, base: T) -> Result<Self, StripPrefixError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(match (self, base.as_strand_view()) {
			(Self::Os(p), Strand::Os(q)) => Self::Os(p.strip_prefix(q)?),
			(Self::Os(p), Strand::Utf8(q)) => Self::Os(p.strip_prefix(q)?),
			(Self::Os(p), Strand::Bytes(b)) => {
				Self::Os(p.strip_prefix(OsStr::from_wtf8(b).map_err(|_| StripPrefixError::WrongEncoding)?)?)
			}
		})
	}
}

impl<'a> PathDyn<'a> {
	#[inline]
	pub fn os<T>(path: &'a T) -> Self
	where
		T: ?Sized + AsRef<std::path::Path>,
	{
		Self::Os(path.as_ref())
	}

	#[inline]
	pub fn as_os(self) -> Result<&'a std::path::Path, PathDynError> {
		match self {
			Self::Os(p) => Ok(p),
		}
	}

	#[inline]
	pub fn to_os_owned(self) -> Result<std::path::PathBuf, PathDynError> {
		match self {
			Self::Os(p) => Ok(p.to_owned()),
		}
	}

	pub fn with<T>(kind: SchemeKind, strand: &'a T) -> Result<Self, StrandError>
	where
		T: ?Sized + AsStrandDyn,
	{
		use SchemeKind as K;

		let s = strand.as_strand_dyn();
		Ok(match kind {
			K::Regular | K::Search | K::Archive => Self::Os(std::path::Path::new(s.as_os()?)),
			K::Sftp => Self::Os(std::path::Path::new(s.as_os()?)), // FIXME
		})
	}

	#[inline]
	pub unsafe fn from_encoded_bytes(kind: impl Into<PathKind>, bytes: &'a [u8]) -> Self {
		match kind.into() {
			PathKind::Os => Self::Os(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) }.as_ref()),
		}
	}
}

// --- PathBufDyn
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
pub enum PathBufDyn {
	Os(std::path::PathBuf),
}

impl From<std::path::PathBuf> for PathBufDyn {
	fn from(value: std::path::PathBuf) -> Self { Self::Os(value) }
}

impl From<&PathBufDyn> for PathBufDyn {
	fn from(value: &PathBufDyn) -> Self { value.clone() }
}

impl TryFrom<PathBufDyn> for std::path::PathBuf {
	type Error = PathBufDynError;

	fn try_from(value: PathBufDyn) -> Result<Self, Self::Error> { value.into_os() }
}

impl PartialEq<PathDyn<'_>> for PathBufDyn {
	fn eq(&self, other: &PathDyn<'_>) -> bool { self.borrow() == *other }
}

impl PartialEq<PathDyn<'_>> for &PathBufDyn {
	fn eq(&self, other: &PathDyn<'_>) -> bool { self.borrow() == *other }
}

impl Equivalent<PathDyn<'_>> for PathBufDyn {
	fn equivalent(&self, key: &PathDyn<'_>) -> bool { self.borrow() == *key }
}

impl PathBufLike for PathBufDyn {
	type Borrowed<'a> = PathDyn<'a>;
	type Strand<'a> = Strand<'a>;

	fn borrow(&self) -> Self::Borrowed<'_> {
		match self {
			Self::Os(p) => Self::Borrowed::Os(p.as_path()),
		}
	}

	fn encoded_bytes(&self) -> &[u8] {
		match self {
			Self::Os(p) => p.as_os_str().as_encoded_bytes(),
		}
	}

	fn into_dyn(self) -> PathBufDyn { self }

	fn into_encoded_bytes(self) -> Vec<u8> {
		match self {
			Self::Os(p) => p.into_os_string().into_encoded_bytes(),
		}
	}

	fn try_set_name<'a, T>(&mut self, name: T) -> Result<(), SetNameError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(match (self, name.as_strand_view()) {
			(Self::Os(p), Strand::Os(q)) => p.set_file_name(q),
			(Self::Os(p), Strand::Utf8(q)) => p.set_file_name(q),
			(Self::Os(p), Strand::Bytes(b)) => {
				p.set_file_name(OsStr::from_wtf8(b).map_err(|_| SetNameError::FromWtf8)?)
			}
		})
	}

	fn try_starts_with<'a, T>(&self, base: T) -> Result<bool, StartsWithError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(match (self, base.as_strand_view()) {
			(Self::Os(p), Strand::Os(q)) => p.starts_with(q),
			(Self::Os(p), Strand::Utf8(q)) => p.starts_with(q),
			(Self::Os(p), Strand::Bytes(b)) => {
				p.starts_with(OsStr::from_wtf8(b).map_err(|_| StartsWithError)?)
			}
		})
	}

	fn take(&mut self) -> Self {
		match self {
			Self::Os(p) => Self::Os(std::mem::take(p)),
		}
	}
}

impl PathBufDyn {
	#[inline]
	pub fn os(path: impl Into<std::path::PathBuf>) -> Self { Self::Os(path.into()) }

	#[inline]
	pub fn into_os(self) -> Result<std::path::PathBuf, PathBufDynError> {
		Ok(match self {
			PathBufDyn::Os(p) => p,
		})
	}

	#[inline]
	pub fn new(kind: SchemeKind) -> Self {
		use SchemeKind as K;

		match kind {
			K::Regular | K::Search | K::Archive => Self::Os(std::path::PathBuf::new()),
			K::Sftp => Self::Os(std::path::PathBuf::new()), // FIXME
		}
	}

	pub fn with<T>(kind: SchemeKind, strand: T) -> Result<Self, StrandError>
	where
		T: AsStrandDyn,
	{
		use SchemeKind as K;

		let s = strand.as_strand_dyn();
		Ok(match kind {
			K::Regular | K::Search | K::Archive => Self::Os(std::path::PathBuf::from(s.as_os()?)),
			K::Sftp => Self::Os(std::path::PathBuf::from(s.as_os()?)), // FIXME
		})
	}

	pub fn with_capacity(kind: SchemeKind, capacity: usize) -> Self {
		use SchemeKind as K;
		match kind {
			K::Regular | K::Search | K::Archive => Self::Os(std::path::PathBuf::with_capacity(capacity)),
			K::Sftp => Self::Os(std::path::PathBuf::with_capacity(capacity)), // FIXME
		}
	}

	pub fn try_push<T>(&mut self, path: T) -> Result<(), StrandError>
	where
		T: AsPathDyn,
	{
		let path = path.as_path_dyn();
		Ok(match self {
			PathBufDyn::Os(p) => p.push(path.as_os()?),
		})
	}

	pub fn try_extend<T>(&mut self, paths: T) -> Result<(), StrandError>
	where
		T: IntoIterator,
		T::Item: AsPathDyn,
	{
		for p in paths {
			self.try_push(p)?;
		}
		Ok(())
	}

	#[inline]
	pub unsafe fn from_encoded_bytes(kind: impl Into<PathKind>, bytes: Vec<u8>) -> Self {
		match kind.into() {
			PathKind::Os => Self::Os(unsafe { std::path::PathBuf::from_encoded_bytes(bytes) }),
		}
	}
}
