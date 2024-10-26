use std::{cmp, ffi::OsStr, fmt::{self, Debug, Formatter}, hash::{Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use super::{Urn, UrnBuf};

#[derive(Clone, Default)]
pub struct Loc {
	path: PathBuf,
	urn:  usize,
	name: usize,
}

unsafe impl Send for Loc {}

unsafe impl Sync for Loc {}

impl Deref for Loc {
	type Target = PathBuf;

	fn deref(&self) -> &Self::Target { &self.path }
}

impl PartialEq for Loc {
	fn eq(&self, other: &Self) -> bool { self.path == other.path }
}

impl Eq for Loc {}

impl Ord for Loc {
	fn cmp(&self, other: &Self) -> cmp::Ordering { self.path.cmp(&other.path) }
}

impl PartialOrd for Loc {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> { Some(self.cmp(other)) }
}

impl Hash for Loc {
	fn hash<H: Hasher>(&self, state: &mut H) { self.path.hash(state) }
}

impl Debug for Loc {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("Loc")
			.field("path", &self.path)
			.field("urn", &self.urn())
			.field("name", &self.name())
			.finish()
	}
}

impl Loc {
	pub fn new(path: PathBuf) -> Self {
		let name = path.file_name().map_or(0, |s| s.len());
		Self { path, urn: name, name }
	}

	pub fn from(base: &Path, path: PathBuf) -> Self {
		let urn = path.strip_prefix(base).unwrap_or(&path).as_os_str().len();
		let name = path.file_name().map_or(0, |s| s.len());
		Self { path, urn, name }
	}

	#[inline]
	pub fn urn(&self) -> &Urn {
		Urn::new(unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(self.bytes().len() - self.urn..),
			)
		})
	}

	#[inline]
	pub fn urn_owned(&self) -> UrnBuf { self.urn().to_owned() }

	#[inline]
	pub fn name(&self) -> &OsStr {
		unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(self.bytes().len() - self.name..),
			)
		}
	}

	#[inline]
	pub fn base(&self) -> &Path {
		Path::new(unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(..self.bytes().len() - self.urn),
			)
		})
	}

	#[inline]
	pub fn rebase(&self, parent: &Path) -> Self {
		debug_assert!(self.urn == self.name);
		let path = parent.join(self.name());

		debug_assert!(path.file_name().is_some_and(|s| s.len() == self.name));
		Self { path, urn: self.name, name: self.name }
	}

	#[inline]
	pub fn into_path(self) -> PathBuf { self.path }

	#[inline]
	fn bytes(&self) -> &[u8] { self.path.as_os_str().as_encoded_bytes() }
}
