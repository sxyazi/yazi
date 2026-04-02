use std::{borrow::Cow, fmt::{self, Debug}};

use crate::{loc::{Loc, LocAble, LocAbleImpl, LocBuf, LocBufAble}, path::{PathBufDyn, PathCow, PathDyn}};

#[derive(Clone)]
pub enum LocCow<'a, B = &'a std::path::Path, O = std::path::PathBuf>
where
	B: LocAble<'a, Owned = O>,
	O: LocBufAble,
{
	Borrowed(Loc<'a, B>),
	Owned(LocBuf<O>),
}

impl<'a, B, O> Debug for LocCow<'a, B, O>
where
	B: LocAble<'a, Owned = O>,
	O: LocBufAble,
	Loc<'a, B>: Debug,
	LocBuf<O>: Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Borrowed(loc) => f.debug_tuple("Borrowed").field(loc).finish(),
			Self::Owned(loc) => f.debug_tuple("Owned").field(loc).finish(),
		}
	}
}

impl<'a, B, O> Default for LocCow<'a, B, O>
where
	B: LocAble<'a, Owned = O> + LocAbleImpl<'a>,
	O: LocBufAble,
{
	fn default() -> Self { Self::Borrowed(Default::default()) }
}

impl<'a, B, O> From<Loc<'a, B>> for LocCow<'a, B, O>
where
	B: LocAble<'a, Owned = O>,
	O: LocBufAble,
{
	fn from(value: Loc<'a, B>) -> Self { Self::Borrowed(value) }
}

impl<'a, B, O> From<LocBuf<O>> for LocCow<'a, B, O>
where
	B: LocAble<'a, Owned = O>,
	O: LocBufAble,
{
	fn from(value: LocBuf<O>) -> Self { Self::Owned(value) }
}

impl<'a, B, O> From<LocCow<'a, B, O>> for PathCow<'a>
where
	B: LocAble<'a, Owned = O> + Into<PathDyn<'a>>,
	O: LocBufAble + Into<PathBufDyn>,
{
	fn from(value: LocCow<'a, B, O>) -> Self { value.into_path() }
}

impl<'a> LocCow<'a> {
	pub fn as_loc(&self) -> Loc<'_> {
		match self {
			Self::Borrowed(loc) => *loc,
			Self::Owned(loc) => loc.as_loc(),
		}
	}

	pub fn into_inner(self) -> Cow<'a, std::path::Path> {
		match self {
			Self::Borrowed(loc) => Cow::Borrowed(loc.as_inner()),
			Self::Owned(loc) => Cow::Owned(loc.into_inner()),
		}
	}
}

impl<'a> LocCow<'a, &'a typed_path::UnixPath, typed_path::UnixPathBuf> {
	pub fn as_loc(&self) -> Loc<'_, &'_ typed_path::UnixPath> {
		match self {
			Self::Borrowed(loc) => *loc,
			Self::Owned(loc) => loc.as_loc(),
		}
	}

	pub fn into_inner(self) -> Cow<'a, typed_path::UnixPath> {
		match self {
			Self::Borrowed(loc) => Cow::Borrowed(loc.as_inner()),
			Self::Owned(loc) => Cow::Owned(loc.into_inner()),
		}
	}
}

impl<'a, B, O> LocCow<'a, B, O>
where
	B: LocAble<'a, Owned = O> + LocAbleImpl<'a>,
	O: LocBufAble,
{
	pub fn into_owned(self) -> LocBuf<O> {
		match self {
			Self::Borrowed(loc) => {
				LocBuf { inner: loc.inner.to_path_buf(), uri: loc.uri, urn: loc.urn }
			}
			Self::Owned(loc) => loc,
		}
	}

	pub fn is_borrowed(&self) -> bool { matches!(self, Self::Borrowed(_)) }

	pub fn is_owned(&self) -> bool { !self.is_borrowed() }
}

impl<'a, B, O> LocCow<'a, B, O>
where
	B: LocAble<'a, Owned = O> + Into<PathDyn<'a>>,
	O: LocBufAble + Into<PathBufDyn>,
{
	pub fn into_path(self) -> PathCow<'a> {
		match self {
			Self::Borrowed(loc) => PathCow::Borrowed(loc.inner.into()),
			Self::Owned(loc) => PathCow::Owned(loc.inner.into()),
		}
	}
}
