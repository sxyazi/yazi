use std::{cmp, ffi::{OsStr, OsString}, fmt::{self, Debug, Formatter}, hash::{Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use crate::url::{Urn, UrnBuf};

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
		let Some(name) = path.file_name() else {
			let urn = path.as_os_str().len();
			return Self { path, urn, name: 0 };
		};

		let name_len = name.len();
		let prefix_len = unsafe {
			name.as_encoded_bytes().as_ptr().offset_from(path.as_os_str().as_encoded_bytes().as_ptr())
		};

		let mut bytes = path.into_os_string().into_encoded_bytes();
		bytes.truncate(name_len + prefix_len as usize);
		Self {
			path: PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(bytes) }),
			urn:  name_len,
			name: name_len,
		}
	}

	pub fn from(base: &Path, path: PathBuf) -> Self {
		let mut loc = Self::new(path);
		loc.urn = loc.path.strip_prefix(base).unwrap_or(&loc.path).as_os_str().len();
		loc
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new() {
		let loc = Loc::new("/".into());
		assert_eq!(loc.urn(), Urn::new("/"));
		assert_eq!(loc.name(), OsStr::new(""));
		assert_eq!(loc.base(), Path::new(""));

		let loc = Loc::new("/root".into());
		assert_eq!(loc.urn(), Urn::new("root"));
		assert_eq!(loc.name(), OsStr::new("root"));
		assert_eq!(loc.base(), Path::new("/"));

		let loc = Loc::new("/root/code/foo/".into());
		assert_eq!(loc.urn(), Urn::new("foo"));
		assert_eq!(loc.name(), OsStr::new("foo"));
		assert_eq!(loc.base(), Path::new("/root/code/"));
	}

	#[test]
	fn test_from() {
		let loc = Loc::from(Path::new("/"), "/".into());
		assert_eq!(loc.urn().as_os_str(), OsStr::new(""));
		assert_eq!(loc.name(), OsStr::new(""));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));

		let loc = Loc::from(Path::new("/root/"), "/root/code/".into());
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.name(), OsStr::new("code"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));

		let loc = Loc::from(Path::new("/root//"), "/root/code/foo//".into());
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.name(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
	}
}
