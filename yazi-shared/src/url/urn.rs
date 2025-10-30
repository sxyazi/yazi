use std::{borrow::Borrow, ops::Deref, path::{Path, PathBuf}};

use serde::Serialize;

use crate::path::{PathBufLike, PathLike};

#[derive(Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Urn<P: ?Sized + PathLike = Path>(P);

impl<P> Urn<P>
where
	P: ?Sized + PathLike,
{
	#[inline]
	pub fn new<T: AsRef<P> + ?Sized>(p: &T) -> &Self {
		unsafe { &*(p.as_ref() as *const P as *const Self) }
	}

	#[inline]
	pub fn name(&self) -> Option<&P::Inner> { self.0.file_name() }

	#[inline]
	pub fn count(&self) -> usize { self.0.components().count() }

	#[inline]
	pub fn encoded_bytes(&self) -> &[u8] { self.0.encoded_bytes() }

	#[cfg(unix)]
	#[inline]
	pub fn is_hidden(&self) -> bool {
		use crate::path::PathInner;
		self.name().is_some_and(|s| s.encoded_bytes().starts_with(b"."))
	}
}

impl<P> Deref for Urn<P>
where
	P: ?Sized + PathLike,
{
	type Target = P;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P> AsRef<P> for Urn<P>
where
	P: ?Sized + PathLike,
{
	fn as_ref(&self) -> &P { &self.0 }
}

impl<P> From<&Urn<P>> for PathBuf
where
	P: ?Sized + PathLike + ToOwned<Owned = PathBuf>,
{
	fn from(value: &Urn<P>) -> Self { value.0.to_owned() }
}

impl<P> ToOwned for Urn<P>
where
	P: ?Sized + PathLike + ToOwned<Owned = <P as PathLike>::Owned>,
	UrnBuf<<P as PathLike>::Owned>: Borrow<Urn<P>>,
{
	type Owned = UrnBuf<<P as PathLike>::Owned>;

	fn to_owned(&self) -> Self::Owned { UrnBuf(self.0.to_owned()) }
}

impl<P> PartialEq<UrnBuf<P::Owned>> for &Urn<P>
where
	P: ?Sized + PathLike + PartialEq<P::Owned>,
{
	fn eq(&self, other: &UrnBuf<P::Owned>) -> bool { self.0 == other.0 }
}

// --- UrnBuf
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize)]
pub struct UrnBuf<P: PathBufLike = PathBuf>(P);

impl<P> Borrow<Urn<P::Borrowed>> for UrnBuf<P>
where
	P: PathBufLike,
{
	fn borrow(&self) -> &Urn<P::Borrowed> { Urn::new(&self.0) }
}

impl<P> Deref for UrnBuf<P>
where
	P: PathBufLike,
{
	type Target = Urn<P::Borrowed>;

	fn deref(&self) -> &Self::Target { Urn::new(&self.0) }
}

impl<P> AsRef<P::Borrowed> for UrnBuf<P>
where
	P: PathBufLike,
{
	fn as_ref(&self) -> &P::Borrowed { Urn::new(&self.0) }
}

impl<P> PartialEq<Urn<P::Borrowed>> for UrnBuf<P>
where
	P: PathBufLike + PartialEq<P::Borrowed>,
{
	fn eq(&self, other: &Urn<P::Borrowed>) -> bool { self.0 == other.0 }
}

impl<T> From<T> for UrnBuf<PathBuf>
where
	T: Into<PathBuf>,
{
	fn from(value: T) -> Self { Self(value.into()) }
}

impl<P> UrnBuf<P>
where
	P: PathBufLike,
{
	#[inline]
	pub fn as_urn(&self) -> &Urn<P::Borrowed> { self }
}
