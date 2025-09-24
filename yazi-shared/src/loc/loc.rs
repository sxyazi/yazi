use std::{ffi::OsStr, hash::{Hash, Hasher}, ops::Deref, path::Path};

use anyhow::{Result, bail};

use crate::{loc::LocBuf, url::{Uri, Urn}};

#[derive(Clone, Copy, Debug)]
pub struct Loc<'a> {
	pub(super) inner: &'a Path,
	pub(super) uri:   usize,
	pub(super) urn:   usize,
}

impl Deref for Loc<'_> {
	type Target = Path;

	fn deref(&self) -> &Self::Target { self.inner }
}

impl AsRef<Path> for Loc<'_> {
	fn as_ref(&self) -> &Path { self.inner }
}

impl<'a> From<&'a LocBuf> for Loc<'a> {
	fn from(value: &'a LocBuf) -> Self {
		Self { inner: &value.inner, uri: value.uri, urn: value.urn }
	}
}

// --- Hash
impl Hash for Loc<'_> {
	fn hash<H: Hasher>(&self, state: &mut H) { self.inner.hash(state) }
}

impl<'a, T: ?Sized + AsRef<OsStr>> From<&'a T> for Loc<'a> {
	fn from(value: &'a T) -> Self {
		let path = Path::new(value.as_ref());
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

		let bytes = path.as_os_str().as_encoded_bytes();
		Self {
			inner: Path::new(unsafe {
				OsStr::from_encoded_bytes_unchecked(&bytes[..prefix_len + name_len])
			}),
			uri:   name_len,
			urn:   name_len,
		}
	}
}

impl From<Loc<'_>> for LocBuf {
	fn from(value: Loc<'_>) -> Self {
		Self { inner: value.inner.to_owned(), uri: value.uri, urn: value.urn }
	}
}

// --- Eq
impl PartialEq for Loc<'_> {
	fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

impl Eq for Loc<'_> {}

impl<'a> Loc<'a> {
	pub fn new<T>(path: &'a T, base: &Path, trail: &Path) -> Self
	where
		T: AsRef<Path> + ?Sized,
	{
		let mut loc = Self::from(path.as_ref());
		loc.uri =
			loc.inner.strip_prefix(base).expect("Loc must start with the given base").as_os_str().len();
		loc.urn =
			loc.inner.strip_prefix(trail).expect("Loc must start with the given trail").as_os_str().len();
		loc
	}

	pub fn with(path: &'a Path, uri: usize, urn: usize) -> Result<Self> {
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

	pub fn zeroed<T: AsRef<Path> + ?Sized>(path: &'a T) -> Self {
		let mut loc = Self::from(path.as_ref());
		(loc.uri, loc.urn) = (0, 0);
		loc
	}

	pub fn floated<T: AsRef<Path> + ?Sized>(path: &'a T, base: &Path) -> Self {
		let mut loc = Self::from(path.as_ref());
		loc.uri =
			loc.inner.strip_prefix(base).expect("Loc must start with the given base").as_os_str().len();
		loc
	}

	#[inline]
	pub fn as_loc(self) -> Self { self }

	#[inline]
	pub fn as_path(self) -> &'a Path { self.inner }

	#[inline]
	pub fn uri(self) -> &'a Uri {
		Uri::new(unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(self.bytes().len() - self.uri..),
			)
		})
	}

	#[inline]
	pub fn urn(self) -> &'a Urn {
		Urn::new(unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(self.bytes().len() - self.urn..),
			)
		})
	}

	#[inline]
	pub fn base(self) -> &'a Urn {
		Urn::new(unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(..self.bytes().len() - self.uri),
			)
		})
	}

	#[inline]
	pub fn has_base(self) -> bool { self.bytes().len() != self.uri }

	#[inline]
	pub fn trail(self) -> &'a Urn {
		Urn::new(unsafe {
			OsStr::from_encoded_bytes_unchecked(
				self.bytes().get_unchecked(..self.bytes().len() - self.urn),
			)
		})
	}

	#[inline]
	pub fn has_trail(self) -> bool { self.bytes().len() != self.urn }

	#[inline]
	pub fn name(self) -> Option<&'a OsStr> { self.inner.file_name() }

	#[inline]
	pub fn stem(self) -> Option<&'a OsStr> { self.inner.file_stem() }

	#[inline]
	pub fn ext(self) -> Option<&'a OsStr> { self.inner.extension() }

	#[inline]
	pub fn parent(self) -> Option<&'a Path> { self.inner.parent() }

	#[inline]
	pub fn triple(self) -> (&'a Path, &'a Path, &'a Path) {
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
	pub fn bytes(self) -> &'a [u8] { self.inner.as_os_str().as_encoded_bytes() }
}
