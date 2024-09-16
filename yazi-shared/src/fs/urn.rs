use std::{borrow::Borrow, path::{Path, PathBuf}};

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct Urn(Path);

impl Urn {
	// TODO: clean this up
	pub fn new<T: AsRef<Path> + ?Sized>(p: &T) -> &Self {
		unsafe { &*(p.as_ref() as *const Path as *const Self) }
	}

	// FIXME: remove this
	pub fn _as_path(&self) -> &Path { &self.0 }
}

impl ToOwned for Urn {
	type Owned = UrnBuf;

	fn to_owned(&self) -> Self::Owned { UrnBuf(self.0.to_owned()) }
}

// --- UrnBuf
pub struct UrnBuf(PathBuf);

impl Borrow<Urn> for UrnBuf {
	fn borrow(&self) -> &Urn { Urn::new(&self.0) }
}

impl UrnBuf {
	// FIXME: remove this
	pub fn _deref(&self) -> &Urn { Urn::new(&self.0) }
}
