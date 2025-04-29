use std::{borrow::{Borrow, Cow}, ffi::OsStr, ops::Deref, path::{Path, PathBuf}};

use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Urn(Path);

impl Urn {
	#[inline]
	pub fn new<T: AsRef<Path> + ?Sized>(p: &T) -> &Self {
		unsafe { &*(p.as_ref() as *const Path as *const Self) }
	}

	#[inline]
	pub fn name(&self) -> Option<&OsStr> { self.0.file_name() }

	#[inline]
	pub fn encoded_bytes(&self) -> &[u8] { self.0.as_os_str().as_encoded_bytes() }

	#[cfg(unix)]
	#[inline]
	pub fn is_hidden(&self) -> bool {
		self.name().is_some_and(|s| s.as_encoded_bytes().starts_with(b"."))
	}
}

impl Deref for Urn {
	type Target = Path;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl AsRef<Path> for Urn {
	fn as_ref(&self) -> &Path { &self.0 }
}

impl ToOwned for Urn {
	type Owned = UrnBuf;

	fn to_owned(&self) -> Self::Owned { UrnBuf(self.0.to_owned()) }
}

impl PartialEq<OsStr> for Urn {
	fn eq(&self, other: &OsStr) -> bool { self.0 == other }
}

impl PartialEq<Cow<'_, OsStr>> for &Urn {
	fn eq(&self, other: &Cow<OsStr>) -> bool { self.0 == other.as_ref() }
}

// --- UrnBuf
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize)]
pub struct UrnBuf(PathBuf);

impl Borrow<Urn> for UrnBuf {
	fn borrow(&self) -> &Urn { Urn::new(&self.0) }
}

impl AsRef<Urn> for UrnBuf {
	fn as_ref(&self) -> &Urn { self.borrow() }
}

impl AsRef<Path> for UrnBuf {
	fn as_ref(&self) -> &Path { &self.0 }
}

impl PartialEq<Urn> for UrnBuf {
	fn eq(&self, other: &Urn) -> bool { self.0 == other.0 }
}

impl<T: Into<PathBuf>> From<T> for UrnBuf {
	fn from(p: T) -> Self { Self(p.into()) }
}

impl UrnBuf {
	#[inline]
	pub fn as_urn(&self) -> &Urn { self.borrow() }
}
