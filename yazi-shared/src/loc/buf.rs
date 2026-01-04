use std::{cmp, ffi::OsStr, fmt::{self, Debug, Formatter}, hash::{Hash, Hasher}, marker::PhantomData, mem, ops::Deref};

use anyhow::Result;

use crate::{loc::{Loc, LocAble, LocAbleImpl, LocBufAble, LocBufAbleImpl}, path::{AsPath, AsPathView, PathDyn, SetNameError}, scheme::SchemeKind, strand::AsStrandView};

#[derive(Clone, Default, Eq, PartialEq)]
pub struct LocBuf<P = std::path::PathBuf> {
	pub(super) inner: P,
	pub(super) uri:   usize,
	pub(super) urn:   usize,
}

impl<P> Deref for LocBuf<P>
where
	P: LocBufAble,
{
	type Target = P;

	fn deref(&self) -> &Self::Target { &self.inner }
}

// FIXME: remove
impl AsRef<std::path::Path> for LocBuf<std::path::PathBuf> {
	fn as_ref(&self) -> &std::path::Path { self.inner.as_ref() }
}

impl<T> AsPath for LocBuf<T>
where
	T: LocBufAble + AsPath,
{
	fn as_path(&self) -> PathDyn<'_> { self.inner.as_path() }
}

impl<T> AsPath for &LocBuf<T>
where
	T: LocBufAble + AsPath,
{
	fn as_path(&self) -> PathDyn<'_> { self.inner.as_path() }
}

impl<P> Ord for LocBuf<P>
where
	P: LocBufAble + Ord,
{
	fn cmp(&self, other: &Self) -> cmp::Ordering { self.inner.cmp(&other.inner) }
}

impl<P> PartialOrd for LocBuf<P>
where
	P: LocBufAble + PartialOrd,
{
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.inner.partial_cmp(&other.inner)
	}
}

// --- Hash
impl<P> Hash for LocBuf<P>
where
	P: LocBufAble + LocBufAbleImpl,
	for<'a> &'a P: AsPathView<'a, P::Borrowed<'a>>,
{
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_loc().hash(state) }
}

impl<P> Debug for LocBuf<P>
where
	P: LocBufAble + LocBufAbleImpl + Debug,
	for<'a> &'a P: AsPathView<'a, P::Borrowed<'a>>,
{
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("LocBuf")
			.field("path", &self.inner)
			.field("uri", &self.uri())
			.field("urn", &self.urn())
			.finish()
	}
}

impl<P> From<P> for LocBuf<P>
where
	P: LocBufAble + LocBufAbleImpl,
	for<'a> &'a P: AsPathView<'a, P::Borrowed<'a>>,
{
	fn from(path: P) -> Self {
		let Loc { inner, uri, urn, _phantom } = Loc::bare(&path);
		let len = inner.len();

		let mut bytes = path.into_encoded_bytes();
		bytes.truncate(len);
		Self { inner: unsafe { P::from_encoded_bytes_unchecked(bytes) }, uri, urn }
	}
}

impl<T: ?Sized + AsRef<OsStr>> From<&T> for LocBuf<std::path::PathBuf> {
	fn from(value: &T) -> Self { Self::from(std::path::PathBuf::from(value)) }
}

impl<P> LocBuf<P>
where
	P: LocBufAble + LocBufAbleImpl,
	for<'a> &'a P: AsPathView<'a, P::Borrowed<'a>>,
{
	pub fn new<'a, S>(path: P, base: S, trail: S) -> Self
	where
		S: for<'b> AsStrandView<'a, <P::Borrowed<'b> as LocAble<'b>>::Strand<'a>>,
	{
		let loc = Self::from(path);
		let Loc { inner, uri, urn, _phantom } = Loc::new(&loc.inner, base, trail);

		debug_assert!(inner.as_encoded_bytes() == loc.inner.as_encoded_bytes());
		Self { inner: loc.inner, uri, urn }
	}

	pub fn with(path: P, uri: usize, urn: usize) -> Result<Self>
	where
		for<'a> P::Borrowed<'a>: LocAble<'a>,
	{
		let loc = Self::from(path);
		let Loc { inner, uri, urn, _phantom } = Loc::with(&loc.inner, uri, urn)?;

		debug_assert!(inner.as_encoded_bytes() == loc.inner.as_encoded_bytes());
		Ok(Self { inner: loc.inner, uri, urn })
	}

	pub fn zeroed<T>(path: T) -> Self
	where
		T: Into<P>,
	{
		let loc = Self::from(path.into());
		let Loc { inner, uri, urn, _phantom } = Loc::zeroed(&loc.inner);

		debug_assert!(inner.as_encoded_bytes() == loc.inner.as_encoded_bytes());
		Self { inner: loc.inner, uri, urn }
	}

	pub fn floated<'a, S>(path: P, base: S) -> Self
	where
		S: for<'b> AsStrandView<'a, <P::Borrowed<'b> as LocAble<'b>>::Strand<'a>>,
	{
		let loc = Self::from(path);
		let Loc { inner, uri, urn, _phantom } = Loc::floated(&loc.inner, base);

		debug_assert!(inner.as_encoded_bytes() == loc.inner.as_encoded_bytes());
		Self { inner: loc.inner, uri, urn }
	}

	pub fn saturated(path: P, kind: SchemeKind) -> Self {
		let loc = Self::from(path);
		let Loc { inner, uri, urn, _phantom } = Loc::saturated(&loc.inner, kind);

		debug_assert!(inner.as_encoded_bytes() == loc.inner.as_encoded_bytes());
		Self { inner: loc.inner, uri, urn }
	}

	#[inline]
	pub fn as_loc<'a>(&'a self) -> Loc<'a, P::Borrowed<'a>> {
		Loc {
			inner:    self.inner.as_path_view(),
			uri:      self.uri,
			urn:      self.urn,
			_phantom: PhantomData,
		}
	}

	#[inline]
	pub fn to_inner(&self) -> P
	where
		P: Clone,
	{
		self.inner.clone()
	}

	#[inline]
	pub fn into_inner(self) -> P { self.inner }

	pub fn try_set_name<'a, T>(&mut self, name: T) -> Result<(), SetNameError>
	where
		T: AsStrandView<'a, P::Strand<'a>>,
	{
		let old = self.inner.len();
		self.mutate(|path| path.set_file_name(name));

		let new = self.inner.len();
		if new == old {
			return Ok(());
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
		Ok(())
	}

	#[inline]
	pub fn rebase<'a, 'b>(&'a self, base: P::Borrowed<'b>) -> Self
	where
		'a: 'b,
		for<'c> <P::Borrowed<'c> as LocAble<'c>>::Owned: Into<Self>,
	{
		let mut loc: Self = base.join(self.uri()).into();
		(loc.uri, loc.urn) = (self.uri, self.urn);
		loc
	}

	#[inline]
	pub fn parent(&self) -> Option<P::Borrowed<'_>> { self.as_loc().parent() }

	#[inline]
	fn mutate<T, F: FnOnce(&mut P) -> T>(&mut self, f: F) -> T {
		let mut inner = mem::take(&mut self.inner);
		let result = f(&mut inner);
		self.inner = Self::from(inner).inner;
		result
	}
}

// FIXME: macro
impl<P> LocBuf<P>
where
	P: LocBufAble + LocBufAbleImpl,
	for<'a> &'a P: AsPathView<'a, P::Borrowed<'a>>,
{
	#[inline]
	pub fn uri(&self) -> P::Borrowed<'_> { self.as_loc().uri() }

	#[inline]
	pub fn urn(&self) -> P::Borrowed<'_> { self.as_loc().urn() }

	#[inline]
	pub fn base(&self) -> P::Borrowed<'_> { self.as_loc().base() }

	#[inline]
	pub fn has_base(&self) -> bool { self.as_loc().has_base() }

	#[inline]
	pub fn trail(&self) -> P::Borrowed<'_> { self.as_loc().trail() }

	#[inline]
	pub fn has_trail(&self) -> bool { self.as_loc().has_trail() }
}

impl LocBuf<std::path::PathBuf> {
	pub const fn empty() -> Self { Self { inner: std::path::PathBuf::new(), uri: 0, urn: 0 } }
}

#[cfg(test)]
mod tests {
	use std::path::{Path, PathBuf};

	use super::*;
	use crate::url::{UrlBuf, UrlLike};

	#[test]
	fn test_new() {
		let loc: LocBuf = Path::new("/").into();
		assert_eq!(loc.uri().as_os_str(), OsStr::new(""));
		assert_eq!(loc.urn().as_os_str(), OsStr::new(""));
		assert_eq!(loc.file_name(), None);
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc: LocBuf = Path::new("/root").into();
		assert_eq!(loc.uri().as_os_str(), OsStr::new("root"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("root"));
		assert_eq!(loc.file_name().unwrap(), OsStr::new("root"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc: LocBuf = Path::new("/root/code/foo/").into();
		assert_eq!(loc.uri().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.file_name().unwrap(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/code/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));
	}

	#[test]
	fn test_with() -> Result<()> {
		let loc = LocBuf::<PathBuf>::with("/".into(), 0, 0)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new(""));
		assert_eq!(loc.urn().as_os_str(), OsStr::new(""));
		assert_eq!(loc.file_name(), None);
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/".into(), 1, 1)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.file_name().unwrap(), OsStr::new("code"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/foo//".into(), 2, 1)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.file_name().unwrap(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/foo//".into(), 2, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.file_name().unwrap(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/foo//bar/".into(), 2, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.file_name().unwrap(), OsStr::new("bar"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/code/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/foo//bar/".into(), 3, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo//bar"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.file_name().unwrap(), OsStr::new("bar"));
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
			a.try_set_name(name).unwrap();
			assert_eq!(
				(a.name(), format!("{a:?}").replace(r"\", "/")),
				(b.name(), expected.replace(r"\", "/"))
			);
		}

		Ok(())
	}
}
