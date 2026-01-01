use std::{borrow::Cow, ffi::OsString, hash::{Hash, Hasher}};

use hashbrown::Equivalent;

use crate::{path::{AsPath, Component, PathDyn, PathDynError, PathKind, SetNameError}, strand::AsStrand, wtf8::FromWtf8Vec};

// --- PathBufDyn
#[derive(Clone, Debug, Eq)]
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

impl From<Cow<'_, std::path::Path>> for PathBufDyn {
	fn from(value: Cow<'_, std::path::Path>) -> Self { Self::Os(value.into_owned()) }
}

impl TryFrom<PathBufDyn> for std::path::PathBuf {
	type Error = PathDynError;

	fn try_from(value: PathBufDyn) -> Result<Self, Self::Error> { value.into_os() }
}

impl TryFrom<PathBufDyn> for typed_path::UnixPathBuf {
	type Error = PathDynError;

	fn try_from(value: PathBufDyn) -> Result<Self, Self::Error> { value.into_unix() }
}

// --- Hash
impl Hash for PathBufDyn {
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_path().hash(state) }
}

// --- PartialEq
impl PartialEq for PathBufDyn {
	fn eq(&self, other: &Self) -> bool { self.as_path() == other.as_path() }
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
		let s = name.as_strand();
		Ok(match self {
			Self::Os(p) => p.set_file_name(s.as_os()?),
			Self::Unix(p) => p.set_file_name(s.encoded_bytes()),
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
