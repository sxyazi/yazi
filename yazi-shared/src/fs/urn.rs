use std::{borrow::{Borrow, Cow}, ffi::OsStr, path::{Path, PathBuf}};

#[derive(Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Urn(Path);

impl Urn {
	// TODO: clean this up
	pub fn new<T: AsRef<Path> + ?Sized>(p: &T) -> &Self {
		unsafe { &*(p.as_ref() as *const Path as *const Self) }
	}

	#[inline]
	pub fn name(&self) -> Option<&OsStr> { self.0.file_name() }

	#[cfg(unix)]
	#[inline]
	pub fn is_hidden(&self) -> bool {
		self.name().map_or(false, |s| s.as_encoded_bytes().starts_with(b"."))
	}

	// FIXME 1: remove this
	pub fn _as_path(&self) -> &Path { &self.0 }
}

impl ToOwned for Urn {
	type Owned = UrnBuf;

	fn to_owned(&self) -> Self::Owned { UrnBuf(self.0.to_owned()) }
}

// --- UrnBuf
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UrnBuf(PathBuf);

impl Borrow<Urn> for UrnBuf {
	fn borrow(&self) -> &Urn { Urn::new(&self.0) }
}

impl PartialEq<Urn> for UrnBuf {
	fn eq(&self, other: &Urn) -> bool { self.0 == other.0 }
}

impl UrnBuf {
	// FIXME 1: remove this
	pub fn _deref(&self) -> &Urn { Urn::new(&self.0) }

	// FIXME 1: remove this
	pub fn _from(p: impl Into<PathBuf>) -> Self { Self(p.into()) }
}
