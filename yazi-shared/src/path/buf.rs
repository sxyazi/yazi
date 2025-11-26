use std::ffi::{OsStr, OsString};

use hashbrown::Equivalent;

use crate::{FromWtf8, FromWtf8Vec, path::{AsPath, Component, PathDyn, PathDynError, PathKind, SetNameError}, strand::{AsStrand, Strand}};

// --- PathBufDyn
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathBufDyn {
	Os(std::path::PathBuf),
	Unix(typed_path::UnixPathBuf),
}

impl From<std::path::PathBuf> for PathBufDyn {
	fn from(value: std::path::PathBuf) -> Self { Self::Os(value) }
}

impl From<typed_path::UnixPathBuf> for PathBufDyn {
	fn from(value: typed_path::UnixPathBuf) -> Self { Self::Unix(value) }
}

impl From<PathDyn<'_>> for PathBufDyn {
	fn from(value: PathDyn<'_>) -> Self { value.to_owned() }
}

impl TryFrom<PathBufDyn> for std::path::PathBuf {
	type Error = PathDynError;

	fn try_from(value: PathBufDyn) -> Result<Self, Self::Error> { value.into_os() }
}

impl TryFrom<PathBufDyn> for typed_path::UnixPathBuf {
	type Error = PathDynError;

	fn try_from(value: PathBufDyn) -> Result<Self, Self::Error> { value.into_unix() }
}

impl PartialEq<PathDyn<'_>> for PathBufDyn {
	fn eq(&self, other: &PathDyn<'_>) -> bool { self.as_path() == *other }
}

impl PartialEq<PathDyn<'_>> for &PathBufDyn {
	fn eq(&self, other: &PathDyn<'_>) -> bool { self.as_path() == *other }
}

impl Equivalent<PathDyn<'_>> for PathBufDyn {
	fn equivalent(&self, key: &PathDyn<'_>) -> bool { self.as_path() == *key }
}

impl PathBufDyn {
	#[inline]
	pub unsafe fn from_encoded_bytes<K>(kind: K, bytes: Vec<u8>) -> Self
	where
		K: Into<PathKind>,
	{
		match kind.into() {
			PathKind::Os => Self::Os(unsafe { OsString::from_encoded_bytes_unchecked(bytes) }.into()),
			PathKind::Unix => Self::Unix(bytes.into()),
		}
	}

	#[inline]
	pub fn from_components<'a, K, I>(kind: K, iter: I) -> Result<Self, PathDynError>
	where
		K: Into<PathKind>,
		I: IntoIterator<Item = Component<'a>>,
	{
		Ok(match kind.into() {
			PathKind::Os => Self::Os(iter.into_iter().collect::<Result<_, PathDynError>>()?),
			PathKind::Unix => Self::Unix(iter.into_iter().collect::<Result<_, PathDynError>>()?),
		})
	}

	pub fn into_encoded_bytes(self) -> Vec<u8> {
		match self {
			Self::Os(p) => p.into_os_string().into_encoded_bytes(),
			Self::Unix(p) => p.into_vec(),
		}
	}

	#[inline]
	pub fn into_os(self) -> Result<std::path::PathBuf, PathDynError> {
		Ok(match self {
			Self::Os(p) => p,
			Self::Unix(_) => Err(PathDynError::AsOs)?,
		})
	}

	#[inline]
	pub fn into_unix(self) -> Result<typed_path::UnixPathBuf, PathDynError> {
		Ok(match self {
			Self::Os(_) => Err(PathDynError::AsUnix)?,
			Self::Unix(p) => p,
		})
	}

	#[inline]
	pub fn new(kind: PathKind) -> Self {
		match kind {
			PathKind::Os => Self::Os(std::path::PathBuf::new()),
			PathKind::Unix => Self::Unix(typed_path::UnixPathBuf::new()),
		}
	}

	pub fn try_extend<T>(&mut self, paths: T) -> Result<(), PathDynError>
	where
		T: IntoIterator,
		T::Item: AsPath,
	{
		for p in paths {
			self.try_push(p)?;
		}
		Ok(())
	}

	pub fn try_push<T>(&mut self, path: T) -> Result<(), PathDynError>
	where
		T: AsPath,
	{
		let path = path.as_path();
		Ok(match self {
			Self::Os(p) => p.push(path.as_os()?),
			Self::Unix(p) => p.push(path.encoded_bytes()),
		})
	}

	pub fn try_set_name<T>(&mut self, name: T) -> Result<(), SetNameError>
	where
		T: AsStrand,
	{
		Ok(match (self, name.as_strand()) {
			(Self::Os(p), Strand::Os(s)) => p.set_file_name(s),
			(Self::Os(p), Strand::Utf8(s)) => p.set_file_name(s),
			(Self::Os(p), Strand::Bytes(b)) => {
				p.set_file_name(OsStr::from_wtf8(b).map_err(|_| SetNameError::FromWtf8)?)
			}

			(Self::Unix(p), s) => p.set_file_name(s.encoded_bytes()),
		})
	}

	pub fn with<K>(kind: K, bytes: Vec<u8>) -> Result<Self, PathDynError>
	where
		K: Into<PathKind>,
	{
		Ok(match kind.into() {
			PathKind::Os => {
				Self::Os(std::path::PathBuf::from_wtf8_vec(bytes).map_err(|_| PathDynError::AsOs)?)
			}
			PathKind::Unix => Self::Unix(bytes.into()),
		})
	}

	pub fn with_capacity<K>(kind: K, capacity: usize) -> Self
	where
		K: Into<PathKind>,
	{
		match kind.into() {
			PathKind::Os => Self::Os(std::path::PathBuf::with_capacity(capacity)),
			PathKind::Unix => Self::Unix(typed_path::UnixPathBuf::with_capacity(capacity)),
		}
	}

	pub fn with_str<K, S>(kind: K, s: S) -> Self
	where
		K: Into<PathKind>,
		S: Into<String>,
	{
		let s = s.into();
		match kind.into() {
			PathKind::Os => Self::Os(s.into()),
			PathKind::Unix => Self::Unix(s.into()),
		}
	}
}
