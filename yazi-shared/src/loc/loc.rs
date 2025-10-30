use std::{hash::{Hash, Hasher}, ops::Deref, path::Path};

use anyhow::{Result, bail};

use crate::{loc::LocBuf, path::{PathInner, PathLike}, url::{Uri, Urn}};

#[derive(Debug)]
pub struct Loc<'a, P: ?Sized + PathLike = Path> {
	pub(super) inner: &'a P,
	pub(super) uri:   usize,
	pub(super) urn:   usize,
}

impl<'a, P> Copy for Loc<'a, P> where P: ?Sized + PathLike {}

impl<'a, P> Clone for Loc<'a, P>
where
	P: ?Sized + PathLike,
{
	fn clone(&self) -> Self { *self }
}

impl<P> Default for Loc<'static, P>
where
	P: ?Sized + PathLike,
{
	fn default() -> Self { Self { inner: P::default(), uri: 0, urn: 0 } }
}

impl<P> Deref for Loc<'_, P>
where
	P: ?Sized + PathLike,
{
	type Target = P;

	fn deref(&self) -> &Self::Target { self.inner }
}

impl<P> AsRef<P> for Loc<'_, P>
where
	P: ?Sized + PathLike,
{
	fn as_ref(&self) -> &P { self.inner }
}

// --- Hash
impl<P> Hash for Loc<'_, P>
where
	P: ?Sized + PathLike + Hash,
{
	fn hash<H: Hasher>(&self, state: &mut H) { self.inner.hash(state) }
}

impl<'a, P, T> From<&'a T> for Loc<'a, P>
where
	P: ?Sized + PathLike,
	T: ?Sized + AsRef<P>,
{
	fn from(value: &'a T) -> Self {
		let path = value.as_ref();
		let Some(name) = path.file_name() else {
			let uri = path.len();
			return Self { inner: path, uri, urn: 0 };
		};

		let name_len = name.len();
		let prefix_len =
			unsafe { name.encoded_bytes().as_ptr().offset_from_unsigned(path.encoded_bytes().as_ptr()) };

		let bytes = path.encoded_bytes();
		Self {
			inner: unsafe { P::from_encoded_bytes(&bytes[..prefix_len + name_len]) },
			uri:   name_len,
			urn:   name_len,
		}
	}
}

impl<P> From<Loc<'_, P>> for LocBuf<<P as PathLike>::Owned>
where
	P: ?Sized + PathLike + ToOwned<Owned = <P as PathLike>::Owned>,
{
	fn from(value: Loc<'_, P>) -> Self {
		Self { inner: value.inner.to_owned(), uri: value.uri, urn: value.urn }
	}
}

// --- Eq
impl<P> PartialEq for Loc<'_, P>
where
	P: ?Sized + PathLike + PartialEq,
{
	fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

impl<P> Eq for Loc<'_, P> where P: ?Sized + PathLike + Eq {}

impl<'a, P> Loc<'a, P>
where
	P: ?Sized + PathLike,
{
	pub fn new<T, U>(path: &'a T, base: &U, trail: &U) -> Self
	where
		T: AsRef<P> + ?Sized,
		U: AsRef<P> + ?Sized,
	{
		let mut loc = Self::from(path);
		loc.uri = loc.inner.strip_prefix(base).expect("Loc must start with the given base").len();
		loc.urn = loc.inner.strip_prefix(trail).expect("Loc must start with the given trail").len();
		loc
	}

	pub fn with<T>(path: &'a T, uri: usize, urn: usize) -> Result<Self>
	where
		T: ?Sized + AsRef<P>,
		<P as PathLike>::Components<'a>: AsRef<P> + Clone + DoubleEndedIterator,
	{
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
				loc.urn = loc.strip_prefix(it.clone()).unwrap().len();
			}
			if i == uri {
				loc.uri = loc.strip_prefix(it).unwrap().len();
				break;
			}
		}
		Ok(loc)
	}

	pub fn zeroed<T>(path: &'a T) -> Self
	where
		T: AsRef<P> + ?Sized,
	{
		let mut loc = Self::from(path);
		(loc.uri, loc.urn) = (0, 0);
		loc
	}

	pub fn floated<T, U>(path: &'a T, base: &U) -> Self
	where
		T: AsRef<P> + ?Sized,
		U: AsRef<P> + ?Sized,
	{
		let mut loc = Self::from(path);
		loc.uri = loc.inner.strip_prefix(base).expect("Loc must start with the given base").len();
		loc
	}

	#[inline]
	pub fn as_loc(self) -> Self { self }

	#[inline]
	pub fn as_path(self) -> &'a P { self.inner }

	#[inline]
	pub fn uri(self) -> &'a Uri<P> {
		Uri::new(unsafe {
			P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(self.inner.len() - self.uri..))
		})
	}

	#[inline]
	pub fn urn(self) -> &'a Urn<P> {
		Urn::new(unsafe {
			P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(self.inner.len() - self.urn..))
		})
	}

	#[inline]
	pub fn base(self) -> &'a Urn<P> {
		Urn::new(unsafe {
			P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(..self.inner.len() - self.uri))
		})
	}

	#[inline]
	pub fn has_base(self) -> bool { self.inner.len() != self.uri }

	#[inline]
	pub fn trail(self) -> &'a Urn<P> {
		Urn::new(unsafe {
			P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(..self.inner.len() - self.urn))
		})
	}

	#[inline]
	pub fn has_trail(self) -> bool { self.inner.len() != self.urn }

	#[inline]
	pub fn name(self) -> Option<&'a P::Inner> { self.inner.file_name() }

	#[inline]
	pub fn stem(self) -> Option<&'a P::Inner> { self.inner.file_stem() }

	#[inline]
	pub fn ext(self) -> Option<&'a P::Inner> { self.inner.extension() }

	#[inline]
	pub fn parent(self) -> Option<&'a P> { self.inner.parent() }

	#[inline]
	pub fn triple(self) -> (&'a P, &'a P, &'a P) {
		let len = self.inner.len();

		let base = ..len - self.uri;
		let rest = len - self.uri..len - self.urn;
		let urn = len - self.urn..;

		unsafe {
			(
				P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(base)),
				P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(rest)),
				P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(urn)),
			)
		}
	}

	#[inline]
	pub fn strip_prefix<T>(self, base: T) -> Option<&'a P>
	where
		T: AsRef<P>,
	{
		self.inner.strip_prefix(base)
	}
}
