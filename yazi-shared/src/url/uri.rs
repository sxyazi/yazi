use std::{ops::Deref, path::Path};

use crate::path::PathLike;

#[derive(Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Uri<P: ?Sized + PathLike = Path>(P);

impl<P> Uri<P>
where
	P: ?Sized + PathLike,
{
	#[inline]
	pub fn new<T: AsRef<P> + ?Sized>(p: &T) -> &Self {
		unsafe { &*(p.as_ref() as *const P as *const Self) }
	}

	#[inline]
	pub fn count(&self) -> usize { self.0.components().count() }

	#[inline]
	pub fn is_empty(&self) -> bool { self.0.len() == 0 }
}

impl<P> Deref for Uri<P>
where
	P: ?Sized + PathLike,
{
	type Target = P;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P> AsRef<P> for Uri<P>
where
	P: ?Sized + PathLike,
{
	fn as_ref(&self) -> &P { &self.0 }
}
