use std::{borrow::Cow, cmp, ffi::{OsStr, OsString}, fmt::{self, Debug, Formatter}, hash::{Hash, Hasher}, ops::Deref, path::{Path, PathBuf}};

use anyhow::{Result, bail};

use crate::url::{Urn, UrnBuf};

#[derive(Clone, Default)]
pub struct Loc {
	inner: PathBuf,
	uri:   usize,
	urn:   usize,
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
			.field("uri", &self.uri())
			.field("urn", &self.urn())
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
			let uri = path.as_os_str().len();
			return Self { inner: path, uri, urn: 0 };
		};

		let name_len = name.len();
		let prefix_len = unsafe {
			name
				.as_encoded_bytes()
				.as_ptr()
				.offset_from_unsigned(path.as_os_str().as_encoded_bytes().as_ptr())
		};

		let mut bytes = path.into_os_string().into_encoded_bytes();
		bytes.truncate(prefix_len + name_len);
		Self {
			inner: PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(bytes) }),
			uri:   name_len,
			urn:   name_len,
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
	pub fn new(path: impl Into<PathBuf>, base: &Path, trail: &Path) -> Self {
		let mut loc = Self::from(path.into());
		loc.uri =
			loc.inner.strip_prefix(base).expect("Loc must start with the given base").as_os_str().len();
		loc.urn =
			loc.inner.strip_prefix(trail).expect("Loc must start with the given trail").as_os_str().len();
		loc
	}

	pub fn with(path: PathBuf, uri: usize, urn: usize) -> Result<Self> {
		if urn > uri {
			bail!("URN cannot be longer than URI");
		}

		let mut loc = Self::from(path);
		if uri == 0 {
			(loc.uri, loc.urn) = (0, 0);
			return Ok(loc);
		} else if urn == 0 {
			loc.urn = 0;
		}

		let mut it = loc.inner.components();
		for i in 1..=uri {
			if it.next_back().is_none() {
				bail!("URI exceeds the entire URL");
			}
			if i == urn {
				loc.urn = loc.strip_prefix(it.clone()).unwrap().as_os_str().len();
			}
			if i == uri {
				loc.uri = loc.strip_prefix(it).unwrap().as_os_str().len();
				break;
			}
		}
		Ok(loc)
	}

	pub fn zeroed(path: impl Into<PathBuf>) -> Self {
		let mut loc = Self::from(path.into());
		(loc.uri, loc.urn) = (0, 0);
		loc
	}

	pub fn floated(path: impl Into<PathBuf>, base: &Path) -> Self {
		let mut loc = Self::from(path.into());
		loc.uri =
			loc.inner.strip_prefix(base).expect("Loc must start with the given base").as_os_str().len();
		loc
	}

	#[inline]
	pub fn uri(&self) -> &Urn {
		Urn::new(unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(self.bytes().len() - self.uri..),
			)
		})
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
	pub fn name(&self) -> &OsStr { self.inner.file_name().unwrap_or(OsStr::new("")) }

	pub fn set_name(&mut self, name: impl AsRef<OsStr>) {
		let (old, new) = (self.name(), name.as_ref());
		if old == new {
			return;
		}

		if old.len() > new.len() {
			let n = old.len() - new.len();
			(self.uri, self.urn) = (self.uri - n, self.urn - n);
		} else {
			let n = new.len() - old.len();
			(self.uri, self.urn) = (self.uri + n, self.urn + n);
		}
		self.inner.set_file_name(new);
	}

	#[inline]
	pub fn base(&self) -> &Urn {
		Urn::new(unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(..self.bytes().len() - self.uri),
			)
		})
	}

	#[inline]
	pub fn has_base(&self) -> bool { self.bytes().len() != self.uri }

	#[inline]
	pub fn trail(&self) -> &Urn {
		Urn::new(unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(..self.bytes().len() - self.urn),
			)
		})
	}

	#[inline]
	pub fn has_trail(&self) -> bool { self.bytes().len() != self.urn }

	#[inline]
	pub fn rebase(&self, parent: &Path) -> Self {
		debug_assert!(self.uri == self.name().len());
		let path = parent.join(self.name());

		debug_assert!(path.file_name().is_some_and(|s| s.len() == self.name().len()));
		Self { inner: path, uri: self.uri, urn: self.uri }
	}

	#[inline]
	pub fn to_path(&self) -> PathBuf { self.inner.clone() }

	#[inline]
	pub fn into_path(self) -> PathBuf { self.inner }

	#[inline]
	pub fn triple(&self) -> (&Path, &Path, &Path) {
		let len = self.bytes().len();

		let base = ..len - self.uri;
		let rest = len - self.uri..len - self.urn;
		let urn = len - self.urn..;

		unsafe {
			(
				Path::new(OsStr::from_encoded_bytes_unchecked(self.bytes().get_unchecked(base))),
				Path::new(OsStr::from_encoded_bytes_unchecked(self.bytes().get_unchecked(rest))),
				Path::new(OsStr::from_encoded_bytes_unchecked(self.bytes().get_unchecked(urn))),
			)
		}
	}

	#[inline]
	fn bytes(&self) -> &[u8] { self.inner.as_os_str().as_encoded_bytes() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new() {
		let loc: Loc = Path::new("/").into();
		assert_eq!(loc.uri().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new(""));
		assert_eq!(loc.name(), OsStr::new(""));
		assert_eq!(loc.base().as_os_str(), OsStr::new(""));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc: Loc = Path::new("/root").into();
		assert_eq!(loc.uri().as_os_str(), OsStr::new("root"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("root"));
		assert_eq!(loc.name(), OsStr::new("root"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc: Loc = Path::new("/root/code/foo/").into();
		assert_eq!(loc.uri().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.name(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/code/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));
	}

	#[test]
	fn test_with() -> Result<()> {
		let loc = Loc::with("/".into(), 0, 0)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new(""));
		assert_eq!(loc.urn().as_os_str(), OsStr::new(""));
		assert_eq!(loc.name(), OsStr::new(""));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc = Loc::with("/root/code/".into(), 1, 1)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.name(), OsStr::new("code"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/"));

		let loc = Loc::with("/root/code/foo//".into(), 2, 1)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.name(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));

		let loc = Loc::with("/root/code/foo//".into(), 2, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.name(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/"));

		let loc = Loc::with("/root/code/foo//bar/".into(), 2, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.name(), OsStr::new("bar"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/code/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));

		let loc = Loc::with("/root/code/foo//bar/".into(), 3, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo//bar"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.name(), OsStr::new("bar"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));
		Ok(())
	}

	#[test]
	fn test_set_name() -> Result<()> {
		const S: char = std::path::MAIN_SEPARATOR;

		let mut loc = Loc::with("/root/code/foo/".into(), 2, 1)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.name(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));

		loc.set_name("bar.txt");
		assert_eq!(loc.uri().as_os_str(), OsString::from(format!("code{S}bar.txt")));
		assert_eq!(loc.name(), OsStr::new("bar.txt"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));

		loc.set_name("baz");
		assert_eq!(loc.uri().as_os_str(), OsString::from(format!("code{S}baz")));
		assert_eq!(loc.name(), OsStr::new("baz"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		Ok(())
	}
}
