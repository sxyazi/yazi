use std::{cmp, ffi::{OsStr, OsString}, fmt::{self, Debug, Formatter}, hash::{Hash, Hasher}, mem, ops::Deref, path::{Path, PathBuf}};

use anyhow::Result;

use crate::{loc::Loc, url::{Uri, Urn}};

#[derive(Clone, Default, Eq)]
pub struct LocBuf {
	pub(super) inner: PathBuf,
	pub(super) uri:   usize,
	pub(super) urn:   usize,
}

impl Deref for LocBuf {
	type Target = PathBuf;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl AsRef<Path> for LocBuf {
	fn as_ref(&self) -> &Path { &self.inner }
}

impl PartialEq for LocBuf {
	fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

impl Ord for LocBuf {
	fn cmp(&self, other: &Self) -> cmp::Ordering { self.inner.cmp(&other.inner) }
}

impl PartialOrd for LocBuf {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> { Some(self.cmp(other)) }
}

// --- Hash
impl Hash for LocBuf {
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_loc().hash(state) }
}

impl Debug for LocBuf {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("LocBuf")
			.field("path", &self.inner)
			.field("uri", &self.uri())
			.field("urn", &self.urn())
			.finish()
	}
}

impl From<PathBuf> for LocBuf {
	fn from(path: PathBuf) -> Self {
		let Loc { inner, uri, urn } = Loc::from(path.as_path());
		let len = inner.as_os_str().len();

		let mut bytes = path.into_os_string().into_encoded_bytes();
		bytes.truncate(len);
		Self {
			inner: PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(bytes) }),
			uri,
			urn,
		}
	}
}

impl<T: ?Sized + AsRef<OsStr>> From<&T> for LocBuf {
	fn from(value: &T) -> Self { Self::from(PathBuf::from(value)) }
}

impl LocBuf {
	pub fn new(path: impl Into<PathBuf>, base: &Path, trail: &Path) -> Self {
		let loc = Self::from(path.into());
		let Loc { inner, uri, urn } = Loc::new(&loc.inner, base, trail);

		debug_assert!(inner.as_os_str() == loc.inner.as_os_str());
		Self { inner: loc.inner, uri, urn }
	}

	pub fn with(path: PathBuf, uri: usize, urn: usize) -> Result<Self> {
		let loc = Self::from(path);
		let Loc { inner, uri, urn } = Loc::with(&loc.inner, uri, urn)?;

		debug_assert!(inner.as_os_str() == loc.inner.as_os_str());
		Ok(Self { inner: loc.inner, uri, urn })
	}

	pub fn zeroed(path: impl Into<PathBuf>) -> Self {
		let loc = Self::from(path.into());
		let Loc { inner, uri, urn } = Loc::zeroed(&loc.inner);

		debug_assert!(inner.as_os_str() == loc.inner.as_os_str());
		Self { inner: loc.inner, uri, urn }
	}

	pub fn floated(path: impl Into<PathBuf>, base: &Path) -> Self {
		let loc = Self::from(path.into());
		let Loc { inner, uri, urn } = Loc::floated(&loc.inner, base);

		debug_assert!(inner.as_os_str() == loc.inner.as_os_str());
		Self { inner: loc.inner, uri, urn }
	}

	#[inline]
	pub fn as_loc<'a>(&'a self) -> Loc<'a> { Loc::from(self) }

	#[inline]
	pub fn to_path(&self) -> PathBuf { self.inner.clone() }

	#[inline]
	pub fn into_path(self) -> PathBuf { self.inner }

	pub fn set_name(&mut self, name: impl AsRef<OsStr>) {
		let old = self.bytes().len();
		self.mutate(|path| path.set_file_name(name));

		let new = self.bytes().len();
		if new == old {
			return;
		}

		if self.uri != 0 {
			if new > old {
				self.uri += new - old;
			} else {
				self.uri -= old - new;
			}
		}
		if self.urn != 0 {
			if new > old {
				self.urn += new - old;
			} else {
				self.urn -= old - new;
			}
		}
	}

	#[inline]
	pub fn rebase(&self, base: &Path) -> Self {
		let mut loc: Self = base.join(self.uri()).into();
		(loc.uri, loc.urn) = (self.uri, self.urn);
		loc
	}

	#[inline]
	fn bytes(&self) -> &[u8] { self.inner.as_os_str().as_encoded_bytes() }

	#[inline]
	fn mutate<F: FnOnce(&mut PathBuf)>(&mut self, f: F) {
		let mut inner = mem::take(&mut self.inner);
		f(&mut inner);
		self.inner = Self::from(inner).inner;
	}
}

// FIXME: macro
impl LocBuf {
	#[inline]
	pub fn uri(&self) -> &Uri { self.as_loc().uri() }

	#[inline]
	pub fn urn(&self) -> &Urn { self.as_loc().urn() }

	#[inline]
	pub fn base(&self) -> &Urn { self.as_loc().base() }

	#[inline]
	pub fn has_base(&self) -> bool { self.as_loc().has_base() }

	#[inline]
	pub fn trail(&self) -> &Urn { self.as_loc().trail() }

	#[inline]
	pub fn has_trail(&self) -> bool { self.as_loc().has_trail() }

	#[inline]
	pub fn name(&self) -> Option<&OsStr> { self.as_loc().name() }

	#[inline]
	pub fn stem(&self) -> Option<&OsStr> { self.as_loc().stem() }

	#[inline]
	pub fn ext(&self) -> Option<&OsStr> { self.as_loc().ext() }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::url::UrlBuf;

	#[test]
	fn test_new() {
		let loc: LocBuf = Path::new("/").into();
		assert_eq!(loc.uri().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new(""));
		assert_eq!(loc.name(), None);
		assert_eq!(loc.base().as_os_str(), OsStr::new(""));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc: LocBuf = Path::new("/root").into();
		assert_eq!(loc.uri().as_os_str(), OsStr::new("root"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("root"));
		assert_eq!(loc.name().unwrap(), OsStr::new("root"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc: LocBuf = Path::new("/root/code/foo/").into();
		assert_eq!(loc.uri().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.name().unwrap(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/code/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));
	}

	#[test]
	fn test_with() -> Result<()> {
		let loc = LocBuf::with("/".into(), 0, 0)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new(""));
		assert_eq!(loc.urn().as_os_str(), OsStr::new(""));
		assert_eq!(loc.name(), None);
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc = LocBuf::with("/root/code/".into(), 1, 1)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.name().unwrap(), OsStr::new("code"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/"));

		let loc = LocBuf::with("/root/code/foo//".into(), 2, 1)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.name().unwrap(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));

		let loc = LocBuf::with("/root/code/foo//".into(), 2, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.name().unwrap(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/"));

		let loc = LocBuf::with("/root/code/foo//bar/".into(), 2, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.name().unwrap(), OsStr::new("bar"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/code/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));

		let loc = LocBuf::with("/root/code/foo//bar/".into(), 3, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo//bar"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.name().unwrap(), OsStr::new("bar"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));
		Ok(())
	}

	#[test]
	fn test_set_name() -> Result<()> {
		crate::init_tests();
		let cases = [
			// Regular
			("/", "a", "/a"),
			("/a/b", "c", "/a/c"),
			// Archive
			("archive:////", "a.zip", "archive:////a.zip"),
			("archive:////a.zip/b", "c", "archive:////a.zip/c"),
			("archive://:2//a.zip/b", "c", "archive://:2//a.zip/c"),
			("archive://:2:1//a.zip/b", "c", "archive://:2:1//a.zip/c"),
			// Empty
			("/a", "", "/"),
			("archive:////a.zip", "", "archive:////"),
			("archive:////a.zip/b", "", "archive:////a.zip"),
			("archive://:1:1//a.zip", "", "archive:////"),
			("archive://:2//a.zip/b", "", "archive://:1//a.zip"),
			("archive://:2:2//a.zip/b", "", "archive://:1:1//a.zip"),
		];

		for (input, name, expected) in cases {
			let mut a: UrlBuf = input.parse()?;
			let b: UrlBuf = expected.parse()?;
			a.set_name(name);
			assert_eq!(
				(a.name(), format!("{a:?}").replace(r"\", "/")),
				(b.name(), expected.replace(r"\", "/"))
			);
		}

		Ok(())
	}
}
