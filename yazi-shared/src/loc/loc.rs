use std::{hash::{Hash, Hasher}, marker::PhantomData, ops::Deref};

use anyhow::{Result, bail};

use super::LocAbleImpl;
use crate::{auth::AuthKind, loc::{LocAble, LocBuf, LocBufAble, StrandAbleImpl}, path::{DynPath, PathDyn, PathView}, strand::AsStrandView};

#[derive(Clone, Copy, Debug)]
pub struct Loc<'p, P = &'p std::path::Path> {
	pub(super) inner:    P,
	pub(super) uri:      usize,
	pub(super) urn:      usize,
	pub(super) _phantom: PhantomData<&'p ()>,
}

impl<'p, P> Default for Loc<'p, P>
where
	P: LocAble<'p> + LocAbleImpl<'p>,
{
	fn default() -> Self { Self { inner: P::empty(), uri: 0, urn: 0, _phantom: PhantomData } }
}

impl<'p, P> Deref for Loc<'p, P>
where
	P: LocAble<'p>,
{
	type Target = P;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl<'p, P> DynPath for Loc<'p, P>
where
	P: LocAble<'p> + DynPath,
{
	fn dyn_path(&self) -> PathDyn<'_> { self.inner.dyn_path() }
}

// FIXME: remove
impl AsRef<std::path::Path> for Loc<'_, &std::path::Path> {
	fn as_ref(&self) -> &std::path::Path { self.inner }
}

// --- Hash
impl<'p, P> Hash for Loc<'p, P>
where
	P: LocAble<'p> + Hash,
{
	fn hash<H: Hasher>(&self, state: &mut H) { self.inner.hash(state) }
}

impl<'p, P> From<Loc<'p, P>> for LocBuf<<P as LocAble<'p>>::Owned>
where
	P: LocAble<'p> + LocAbleImpl<'p>,
	<P as LocAble<'p>>::Owned: LocBufAble,
{
	fn from(value: Loc<'p, P>) -> Self {
		Self { inner: value.inner.to_path_buf(), uri: value.uri, urn: value.urn }
	}
}

// --- Eq
impl<'p, P> PartialEq for Loc<'p, P>
where
	P: LocAble<'p> + PartialEq,
{
	fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

impl<'p, P> Eq for Loc<'p, P> where P: LocAble<'p> + Eq {}

impl<'p, P> Loc<'p, P>
where
	P: LocAble<'p> + LocAbleImpl<'p>,
{
	#[inline]
	pub fn as_inner(self) -> P { self.inner }

	#[inline]
	pub fn as_loc(self) -> Self { self }

	pub fn bare<T>(path: T) -> Self
	where
		T: PathView<'p, P>,
	{
		let path = path.path_view();
		let Some(name) = path.file_name() else {
			let p = path.strip_prefix(P::empty()).unwrap();
			return Self { inner: p, uri: 0, urn: 0, _phantom: PhantomData };
		};

		let name_len = name.len();
		let prefix_len = unsafe {
			name.as_encoded_bytes().as_ptr().offset_from_unsigned(path.as_encoded_bytes().as_ptr())
		};

		let bytes = &path.as_encoded_bytes()[..prefix_len + name_len];
		Self {
			inner:    unsafe { P::from_encoded_bytes_unchecked(bytes) },
			uri:      name_len,
			urn:      name_len,
			_phantom: PhantomData,
		}
	}

	#[inline]
	pub fn base(self) -> P {
		unsafe {
			P::from_encoded_bytes_unchecked(
				self.inner.as_encoded_bytes().get_unchecked(..self.inner.len() - self.uri),
			)
		}
	}

	pub fn floated<'a, T, S>(path: T, base: S) -> Self
	where
		T: PathView<'p, P>,
		S: AsStrandView<'a, P::Strand<'a>>,
	{
		let mut loc = Self::bare(path);
		loc.uri = loc.inner.strip_prefix(base).expect("Loc must start with the given base").len();
		loc
	}

	#[inline]
	pub fn has_base(self) -> bool { self.inner.len() != self.uri }

	#[inline]
	pub fn has_trail(self) -> bool { self.inner.len() != self.urn }

	#[inline]
	pub fn is_empty(self) -> bool { self.inner.len() == 0 }

	pub fn new<'a, T, S>(path: T, base: S, trail: S) -> Self
	where
		T: PathView<'p, P>,
		S: AsStrandView<'a, P::Strand<'a>>,
	{
		let mut loc = Self::bare(path);
		loc.uri = loc.inner.strip_prefix(base).expect("Loc must start with the given base").len();
		loc.urn = loc.inner.strip_prefix(trail).expect("Loc must start with the given trail").len();
		loc
	}

	#[inline]
	pub fn parent(self) -> Option<P> { self.inner.parent() }

	pub fn saturated<'a, T>(path: T, kind: AuthKind) -> Self
	where
		T: PathView<'p, P>,
	{
		match kind {
			AuthKind::Regular => Self::bare(path),
			AuthKind::Search => Self::zeroed(path),
			AuthKind::Mount => Self::zeroed(path),
			AuthKind::Hub => Self::bare(path),
			AuthKind::Scope => Self::bare(path),
			AuthKind::Sftp => Self::bare(path),
		}
	}

	#[inline]
	pub fn trail(self) -> P {
		unsafe {
			P::from_encoded_bytes_unchecked(
				self.inner.as_encoded_bytes().get_unchecked(..self.inner.len() - self.urn),
			)
		}
	}

	#[inline]
	pub fn triple(self) -> (P, P, P) {
		let len = self.inner.len();

		let base = ..len - self.uri;
		let rest = len - self.uri..len - self.urn;
		let urn = len - self.urn..;

		unsafe {
			(
				P::from_encoded_bytes_unchecked(self.inner.as_encoded_bytes().get_unchecked(base)),
				P::from_encoded_bytes_unchecked(self.inner.as_encoded_bytes().get_unchecked(rest)),
				P::from_encoded_bytes_unchecked(self.inner.as_encoded_bytes().get_unchecked(urn)),
			)
		}
	}

	#[inline]
	pub fn uri(self) -> P {
		unsafe {
			P::from_encoded_bytes_unchecked(
				self.inner.as_encoded_bytes().get_unchecked(self.inner.len() - self.uri..),
			)
		}
	}

	#[inline]
	pub fn urn(self) -> P {
		unsafe {
			P::from_encoded_bytes_unchecked(
				self.inner.as_encoded_bytes().get_unchecked(self.inner.len() - self.urn..),
			)
		}
	}

	pub fn with<T>(path: T, uri: usize, urn: usize) -> Result<Self>
	where
		T: PathView<'p, P>,
	{
		if urn > uri {
			bail!("URN cannot be longer than URI");
		}

		let mut loc = Self::bare(path);
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
				loc.urn = loc.strip_prefix(it.clone()).unwrap().len();
			}
			if i == uri {
				loc.uri = loc.strip_prefix(it).unwrap().len();
				break;
			}
		}
		Ok(loc)
	}

	pub fn zeroed<T>(path: T) -> Self
	where
		T: PathView<'p, P>,
	{
		let mut loc = Self::bare(path);
		(loc.uri, loc.urn) = (0, 0);
		loc
	}
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use typed_path::UnixPath;

	use super::*;

	#[test]
	fn test_parent() {
		assert_eq!(Loc::bare(Path::new("foo")).parent(), Some(Path::new("")));
		assert_eq!(Loc::bare(Path::new("")).parent(), None);

		assert_eq!(Loc::bare(UnixPath::new("foo")).parent(), Some(UnixPath::new("")));
		assert_eq!(Loc::bare(UnixPath::new("")).parent(), None);
	}

	#[test]
	fn test_with() -> Result<()> {
		let cases = [
			// Relative paths
			("tmp/test.zip/foo/bar", 3, 2, "test.zip/foo/bar", "foo/bar"),
			("tmp/test.zip/foo/bar/", 3, 2, "test.zip/foo/bar", "foo/bar"),
			// Absolute paths
			("/tmp/test.zip/foo/bar", 3, 2, "test.zip/foo/bar", "foo/bar"),
			("/tmp/test.zip/foo/bar/", 3, 2, "test.zip/foo/bar", "foo/bar"),
			// Relative path with parent components
			("tmp/test.zip/foo/bar/../..", 5, 4, "test.zip/foo/bar/../..", "foo/bar/../.."),
			("tmp/test.zip/foo/bar/../../", 5, 4, "test.zip/foo/bar/../..", "foo/bar/../.."),
			// Absolute path with parent components
			("/tmp/test.zip/foo/bar/../..", 5, 4, "test.zip/foo/bar/../..", "foo/bar/../.."),
			("/tmp/test.zip/foo/bar/../../", 5, 4, "test.zip/foo/bar/../..", "foo/bar/../.."),
		];

		for (path, uri, urn, expect_uri, expect_urn) in cases {
			let loc = Loc::with(Path::new(path), uri, urn)?;
			assert_eq!(loc.uri().to_str().unwrap(), expect_uri);
			assert_eq!(loc.urn().to_str().unwrap(), expect_urn);

			let loc = Loc::with(UnixPath::new(path), uri, urn)?;
			assert_eq!(loc.uri().to_str().unwrap(), expect_uri);
			assert_eq!(loc.urn().to_str().unwrap(), expect_urn);
		}

		Ok(())
	}
}
