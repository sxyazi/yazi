use std::{cmp, ffi::OsStr, fmt::{self, Debug, Formatter}, hash::{Hash, Hasher}, ops::Deref, path::PathBuf};

use anyhow::Result;

use crate::{loc::Loc, path::{PathBufLike, PathLike}, url::{Uri, Urn}};

#[derive(Clone, Default, Eq)]
pub struct LocBuf<P: PathBufLike = PathBuf> {
	pub(super) inner: P,
	pub(super) uri:   usize,
	pub(super) urn:   usize,
}

impl<P> Deref for LocBuf<P>
where
	P: PathBufLike,
{
	type Target = P;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl<P> AsRef<P::Borrowed> for LocBuf<P>
where
	P: PathBufLike,
{
	fn as_ref(&self) -> &P::Borrowed { self.inner.as_ref() }
}

impl<P> PartialEq for LocBuf<P>
where
	P: PathBufLike + PartialEq,
{
	fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

impl<P> Ord for LocBuf<P>
where
	P: PathBufLike + Ord,
{
	fn cmp(&self, other: &Self) -> cmp::Ordering { self.inner.cmp(&other.inner) }
}

impl<P> PartialOrd for LocBuf<P>
where
	P: PathBufLike + PartialOrd,
{
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.inner.partial_cmp(&other.inner)
	}
}

// --- Hash
impl<P> Hash for LocBuf<P>
where
	P: PathBufLike,
	P::Borrowed: Hash,
{
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_loc().hash(state) }
}

impl<P> Debug for LocBuf<P>
where
	P: PathBufLike + Debug,
	P::Borrowed: Debug,
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
	P: PathBufLike,
{
	fn from(path: P) -> Self {
		let Loc { inner, uri, urn } = Loc::from(&path);
		let len = inner.len();

		let mut bytes = path.into_encoded_bytes();
		bytes.truncate(len);
		Self { inner: unsafe { P::from_encoded_bytes(bytes) }, uri, urn }
	}
}

impl<T: ?Sized + AsRef<OsStr>> From<&T> for LocBuf<PathBuf> {
	fn from(value: &T) -> Self { Self::from(PathBuf::from(value)) }
}

impl<P> LocBuf<P>
where
	P: PathBufLike,
{
	pub fn new<T>(path: P, base: &T, trail: &T) -> Self
	where
		T: AsRef<P::Borrowed> + ?Sized,
	{
		let loc = Self::from(path);
		let Loc { inner, uri, urn } = Loc::new(&loc.inner, base, trail);

		debug_assert!(inner.encoded_bytes() == loc.inner.encoded_bytes());
		Self { inner: loc.inner, uri, urn }
	}

	pub fn with(path: P, uri: usize, urn: usize) -> Result<Self>
	where
		P::Borrowed: PathLike,
	{
		let loc = Self::from(path);
		let Loc { inner, uri, urn } = Loc::with(&loc.inner, uri, urn)?;

		debug_assert!(inner.encoded_bytes() == loc.inner.encoded_bytes());
		Ok(Self { inner: loc.inner, uri, urn })
	}

	// FIXME: use `LocBuf::empty()` when Rust 1.91.0 released
	// pub const fn empty() -> Self { Self { inner: PathBuf::new(), uri: 0, urn: 0 }
	// }

	pub fn zeroed<T>(path: T) -> Self
	where
		T: Into<P>,
	{
		let loc = Self::from(path.into());
		let Loc { inner, uri, urn } = Loc::zeroed(&loc.inner);

		debug_assert!(inner.encoded_bytes() == loc.inner.encoded_bytes());
		Self { inner: loc.inner, uri, urn }
	}

	pub fn floated<T, U>(path: T, base: &U) -> Self
	where
		T: Into<P>,
		U: AsRef<P::Borrowed> + ?Sized,
	{
		let loc = Self::from(path.into());
		let Loc { inner, uri, urn } = Loc::floated(&loc.inner, base);

		debug_assert!(inner.encoded_bytes() == loc.inner.encoded_bytes());
		Self { inner: loc.inner, uri, urn }
	}

	#[inline]
	pub fn as_loc<'a>(&'a self) -> Loc<'a, P::Borrowed> {
		Loc { inner: self.inner.as_ref(), uri: self.uri, urn: self.urn }
	}

	#[inline]
	pub fn to_path(&self) -> P
	where
		P: Clone,
	{
		self.inner.clone()
	}

	#[inline]
	pub fn into_path(self) -> P { self.inner }

	pub fn set_name<T>(&mut self, name: T)
	where
		T: AsRef<P::InnerRef>,
	{
		let old = self.inner.len();
		self.mutate(|path| path.set_file_name(name));

		let new = self.len();
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
	pub fn rebase(&self, base: &P::Borrowed) -> Self
	where
		<P::Borrowed as PathLike>::Owned: Into<Self>,
	{
		let mut loc: Self = base.join(self.uri()).into();
		(loc.uri, loc.urn) = (self.uri, self.urn);
		loc
	}

	#[inline]
	fn mutate<F: FnOnce(&mut P)>(&mut self, f: F)
	where
		P: Default,
	{
		let mut inner = std::mem::take(&mut self.inner);
		f(&mut inner);
		self.inner = Self::from(inner).inner;
	}
}

// FIXME: macro
impl<P> LocBuf<P>
where
	P: PathBufLike,
{
	#[inline]
	pub fn uri(&self) -> &Uri<P::Borrowed> { self.as_loc().uri() }

	#[inline]
	pub fn urn(&self) -> &Urn<P::Borrowed> { self.as_loc().urn() }

	#[inline]
	pub fn base(&self) -> &Urn<P::Borrowed> { self.as_loc().base() }

	#[inline]
	pub fn has_base(&self) -> bool { self.as_loc().has_base() }

	#[inline]
	pub fn trail(&self) -> &Urn<P::Borrowed> { self.as_loc().trail() }

	#[inline]
	pub fn has_trail(&self) -> bool { self.as_loc().has_trail() }

	#[inline]
	pub fn name(&self) -> Option<&<P::Borrowed as PathLike>::Inner> { self.as_loc().name() }

	#[inline]
	pub fn stem(&self) -> Option<&<P::Borrowed as PathLike>::Inner> { self.as_loc().stem() }

	#[inline]
	pub fn ext(&self) -> Option<&<P::Borrowed as PathLike>::Inner> { self.as_loc().ext() }
}

#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;
	use crate::url::{UrlBuf, UrlLike};

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
		let loc = LocBuf::<PathBuf>::with("/".into(), 0, 0)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new(""));
		assert_eq!(loc.urn().as_os_str(), OsStr::new(""));
		assert_eq!(loc.name(), None);
		assert_eq!(loc.base().as_os_str(), OsStr::new("/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/".into(), 1, 1)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code"));
		assert_eq!(loc.name().unwrap(), OsStr::new("code"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/foo//".into(), 2, 1)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo"));
		assert_eq!(loc.name().unwrap(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/foo//".into(), 2, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("code/foo"));
		assert_eq!(loc.name().unwrap(), OsStr::new("foo"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/foo//bar/".into(), 2, 2)?;
		assert_eq!(loc.uri().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.urn().as_os_str(), OsStr::new("foo//bar"));
		assert_eq!(loc.name().unwrap(), OsStr::new("bar"));
		assert_eq!(loc.base().as_os_str(), OsStr::new("/root/code/"));
		assert_eq!(loc.trail().as_os_str(), OsStr::new("/root/code/"));

		let loc = LocBuf::<PathBuf>::with("/root/code/foo//bar/".into(), 3, 2)?;
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
