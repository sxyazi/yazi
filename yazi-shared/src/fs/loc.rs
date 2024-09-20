use std::{cmp, ffi::OsStr, fmt::{self, Debug, Formatter}, hash::{Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use super::{Urn, UrnBuf};

pub struct Loc {
	path: PathBuf,
	urn:  *const OsStr,
	name: *const OsStr,
}

unsafe impl Send for Loc {}

unsafe impl Sync for Loc {}

impl Default for Loc {
	fn default() -> Self {
		Self { path: PathBuf::default(), urn: OsStr::new(""), name: OsStr::new("") }
	}
}

impl Clone for Loc {
	fn clone(&self) -> Self {
		let path = self.path.clone();
		let name = path.file_name().unwrap_or(OsStr::new("")) as *const OsStr;
		let urn = if self.urn()._as_path() == self.name() { name } else { self.twin_urn(&path) };
		Self { path, urn, name }
	}
}

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
		let urn = path.file_name().unwrap_or(OsStr::new("")) as *const OsStr;
		Self { path, urn, name: urn }
	}

	pub fn from(base: &Path, path: PathBuf) -> Self {
		let urn = path.strip_prefix(base).unwrap_or(&path).as_os_str() as *const OsStr;
		let name = path.file_name().unwrap_or(OsStr::new("")) as *const OsStr;
		Self { path, urn, name }
	}

	pub fn base(&self) -> &Path {
		let mut it = self.path.components();
		for _ in 0..self.urn()._as_path().components().count() {
			it.next_back().unwrap();
		}
		it.as_path()
	}

	pub fn rebase(&self, parent: &Path) -> Self {
		debug_assert!(self.urn()._as_path() == self.name());

		let path = parent.join(self.name());
		let name = path.file_name().unwrap_or(OsStr::new("")) as *const OsStr;
		Self { path, urn: name, name }
	}

	#[inline]
	fn twin_urn<'a>(&self, new: &'a Path) -> &'a OsStr {
		let total = new.components().count();
		let take = self.urn()._as_path().components().count();

		let mut it = new.components();
		for _ in 0..total - take {
			it.next().unwrap();
		}

		it.as_path().as_os_str()
	}
}

impl Loc {
	#[inline]
	pub fn urn(&self) -> &Urn { Urn::new(unsafe { &*self.urn }) }

	#[inline]
	pub fn urn_owned(&self) -> UrnBuf { self.urn().to_owned() }

	#[inline]
	pub fn name(&self) -> &OsStr { unsafe { &*self.name } }

	#[inline]
	pub fn into_path(self) -> PathBuf { self.path }
}
