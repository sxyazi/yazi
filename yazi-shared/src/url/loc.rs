use std::{borrow::Cow, cmp, ffi::{OsStr, OsString}, fmt::{self, Debug, Formatter}, hash::{Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use crate::url::{Urn, UrnBuf};

#[derive(Clone, Default)]
pub struct Loc {
	inner: PathBuf,
	urn:   usize,
	name:  usize,
}

impl Deref for Loc {
	type Target = PathBuf;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl AsRef<Path> for Loc {
	fn as_ref(&self) -> &Path { &self.inner }
}

impl PartialEq for Loc {
	fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

impl Eq for Loc {}

impl Ord for Loc {
	fn cmp(&self, other: &Self) -> cmp::Ordering { self.inner.cmp(&other.inner) }
}

impl PartialOrd for Loc {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> { Some(self.cmp(other)) }
}

impl Hash for Loc {
	fn hash<H: Hasher>(&self, state: &mut H) { self.inner.hash(state) }
}

impl Debug for Loc {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("Loc")
			.field("path", &self.inner)
			.field("urn", &self.urn())
			.field("name", &self.name())
			.finish()
	}
}

impl From<OsString> for Loc {
	fn from(value: OsString) -> Self { Self::from(PathBuf::from(value)) }
}

impl From<String> for Loc {
	fn from(value: String) -> Self { Self::from(PathBuf::from(value)) }
}

impl From<PathBuf> for Loc {
	fn from(path: PathBuf) -> Self {
		let Some(name) = path.file_name() else {
			let urn = path.as_os_str().len();
			return Self { inner: path, urn, name: 0 };
		};

		let name_len = name.len();
		let prefix_len = unsafe {
			name.as_encoded_bytes().as_ptr().offset_from(path.as_os_str().as_encoded_bytes().as_ptr())
		};

		let mut bytes = path.into_os_string().into_encoded_bytes();
		bytes.truncate(name_len + prefix_len as usize);
		Self {
			inner: PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(bytes) }),
			urn:   name_len,
			name:  name_len,
		}
	}
}

impl From<Cow<'_, Path>> for Loc {
	fn from(value: Cow<'_, Path>) -> Self { Self::from(value.into_owned()) }
}

impl<T: ?Sized + AsRef<OsStr>> From<&T> for Loc {
	fn from(value: &T) -> Self { Self::from(value.as_ref().to_os_string()) }
}

impl Loc {
	pub fn with(base: &Path, path: PathBuf) -> Self {
		let mut loc = Self::from(path);
		loc.urn = loc.inner.strip_prefix(base).unwrap_or(&loc.inner).as_os_str().len();
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

	pub fn set_name(&mut self, name: impl AsRef<OsStr>) {
		let name = name.as_ref();
		if name == self.name() {
			return;
		}

		if self.name > name.len() {
			self.urn -= self.name - name.len();
		} else {
			self.urn += name.len() - self.name;
		}

		self.name = name.len();
		self.inner.set_file_name(name);
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
	pub fn has_base(&self) -> bool { self.bytes().len() != self.urn }

	#[inline]
	pub fn rebase(&self, parent: &Path) -> Self {
		debug_assert!(self.urn == self.name);
		let path = parent.join(self.name());

		debug_assert!(path.file_name().is_some_and(|s| s.len() == self.name));
		Self { inner: path, urn: self.name, name: self.name }
	}

	#[inline]
	pub fn into_path(self) -> PathBuf { self.inner }

	#[inline]
	fn bytes(&self) -> &[u8] { self.inner.as_os_str().as_encoded_bytes() }
}

#[cfg(test)]
mod tests {
	use std::path::MAIN_SEPARATOR;

	use super::*;

	#[test]
	fn test_new() {
		let loc: Loc = Path::new("/").into();
		assert_eq!(loc.urn(), Urn::new("/"));
		assert_eq!(loc.name(), OsStr::new(""));
		assert_eq!(loc.base(), Path::new(""));

		let loc: Loc = Path::new("/root").into();
		assert_eq!(loc.urn(), Urn::new("root"));
		assert_eq!(loc.name(), OsStr::new("root"));
		assert_eq!(loc.base(), Path::new("/"));

		let loc: Loc = Path::new("/root/code/foo/").into();
		assert_eq!(loc.urn(), Urn::new("foo"));
		assert_eq!(loc.name(), OsStr::new("foo"));
		assert_eq!(loc.base(), Path::new("/root/code/"));
	}

	#[test]
	fn test_from() {
		let loc = Loc::with(Path::new("/"), "/".into());
		assert_eq!(loc.urn().as_os_str(), OsStr::new(""));
		assert_eq!(loc.name(), OsStr::new(""));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));

		let loc = Loc::with(Path::new("/root/"), "/root/code/".into());
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.name(), OsStr::new("code"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));

		let loc = Loc::with(Path::new("/root//"), "/root/code/foo//".into());
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.name(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
	}

	#[test]
	fn test_set_name() {
		let mut loc = Loc::with(Path::new("/root"), "/root/code/foo/".into());
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.name(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));

		loc.set_name("bar.txt");
		assert_eq!(loc.urn().as_os_str(), OsString::from(format!("code{MAIN_SEPARATOR}bar.txt")));
		assert_eq!(loc.name(), OsStr::new("bar.txt"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));

		loc.set_name("baz");
		assert_eq!(loc.urn().as_os_str(), OsString::from(format!("code{MAIN_SEPARATOR}baz")));
		assert_eq!(loc.name(), OsStr::new("baz"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
	}
}
