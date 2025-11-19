use std::ffi::{OsStr, OsString};

use hashbrown::Equivalent;
use serde::Serialize;

use crate::{FromWtf8, path::{AsPath, PathBufDynError, PathDyn, PathKind, SetNameError}, strand::{AsStrand, Strand, StrandError}};

// --- PathBufDyn
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
pub enum PathBufDyn {
	Os(std::path::PathBuf),
}

impl From<std::path::PathBuf> for PathBufDyn {
	fn from(value: std::path::PathBuf) -> Self { Self::Os(value) }
}

impl From<PathDyn<'_>> for PathBufDyn {
	fn from(value: PathDyn<'_>) -> Self { value.to_owned() }
}

impl TryFrom<PathBufDyn> for std::path::PathBuf {
	type Error = PathBufDynError;

	fn try_from(value: PathBufDyn) -> Result<Self, Self::Error> { value.into_os() }
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
		}
	}

	pub fn into_encoded_bytes(self) -> Vec<u8> {
		match self {
			Self::Os(p) => p.into_os_string().into_encoded_bytes(),
		}
	}

	#[inline]
	pub fn into_os(self) -> Result<std::path::PathBuf, PathBufDynError> {
		Ok(match self {
			Self::Os(p) => p,
		})
	}

	#[inline]
	pub fn new(kind: PathKind) -> Self {
		match kind {
			PathKind::Os => Self::Os(std::path::PathBuf::new()),
		}
	}

	#[inline]
	pub fn os(path: impl Into<std::path::PathBuf>) -> Self { Self::Os(path.into()) }

	pub fn try_extend<T>(&mut self, paths: T) -> Result<(), StrandError>
	where
		T: IntoIterator,
		T::Item: AsPath,
	{
		for p in paths {
			self.try_push(p)?;
		}
		Ok(())
	}

	pub fn try_push<T>(&mut self, path: T) -> Result<(), StrandError>
	where
		T: AsPath,
	{
		let path = path.as_path();
		Ok(match self {
			Self::Os(p) => p.push(path.as_os()?),
		})
	}

	pub fn try_set_name<T>(&mut self, name: T) -> Result<(), SetNameError>
	where
		T: AsStrand,
	{
		Ok(match (self, name.as_strand()) {
			(Self::Os(p), Strand::Os(q)) => p.set_file_name(q),
			(Self::Os(p), Strand::Utf8(q)) => p.set_file_name(q),
			(Self::Os(p), Strand::Bytes(b)) => {
				p.set_file_name(OsStr::from_wtf8(b).map_err(|_| SetNameError::FromWtf8)?)
			}
		})
	}

	pub fn with<K, T>(kind: K, strand: T) -> Result<Self, StrandError>
	where
		K: Into<PathKind>,
		T: AsStrand,
	{
		let s = strand.as_strand();
		Ok(match kind.into() {
			PathKind::Os => Self::Os(std::path::PathBuf::from(s.as_os()?)),
		})
	}

	pub fn with_capacity<K>(kind: K, capacity: usize) -> Self
	where
		K: Into<PathKind>,
	{
		match kind.into() {
			PathKind::Os => Self::Os(std::path::PathBuf::with_capacity(capacity)),
		}
	}
}
