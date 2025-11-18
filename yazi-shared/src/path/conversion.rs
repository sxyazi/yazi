use std::{borrow::Cow, ffi::OsStr};

use super::{PathBufDyn, PathDyn};
use crate::path::{PathBufLike, PathCow, PathLike};

// --- AsPath
pub trait AsPath {
	fn as_path(&self) -> impl PathLike<'_>;
}

impl AsPath for OsStr {
	fn as_path(&self) -> impl PathLike<'_> { std::path::Path::new(self) }
}

impl AsPath for &OsStr {
	fn as_path(&self) -> impl PathLike<'_> { std::path::Path::new(self) }
}

impl AsPath for std::path::Path {
	fn as_path(&self) -> impl PathLike<'_> { self }
}

impl AsPath for std::path::PathBuf {
	fn as_path(&self) -> impl PathLike<'_> { self.as_path() }
}

impl AsPath for PathDyn<'_> {
	fn as_path(&self) -> impl PathLike<'_> { *self }
}

impl AsPath for PathBufDyn {
	fn as_path(&self) -> impl PathLike<'_> { self.borrow() }
}

impl AsPath for &PathBufDyn {
	fn as_path(&self) -> impl PathLike<'_> { self.borrow() }
}

impl AsPath for PathCow<'_> {
	fn as_path(&self) -> impl PathLike<'_> {
		match self {
			PathCow::Borrowed(p) => *p,
			PathCow::Owned(p) => p.into(),
		}
	}
}

// --- AsPathDyn
pub trait AsPathDyn {
	fn as_path_dyn(&self) -> PathDyn<'_>;
}

impl AsPathDyn for OsStr {
	fn as_path_dyn(&self) -> PathDyn<'_> { std::path::Path::new(self).into() }
}

impl AsPathDyn for &OsStr {
	fn as_path_dyn(&self) -> PathDyn<'_> { std::path::Path::new(self).into() }
}

impl AsPathDyn for Cow<'_, OsStr> {
	fn as_path_dyn(&self) -> PathDyn<'_> { std::path::Path::new(self).into() }
}

impl AsPathDyn for std::path::Path {
	fn as_path_dyn(&self) -> PathDyn<'_> { self.into() }
}

impl AsPathDyn for std::path::PathBuf {
	fn as_path_dyn(&self) -> PathDyn<'_> { self.as_path().into() }
}

impl AsPathDyn for &std::path::PathBuf {
	fn as_path_dyn(&self) -> PathDyn<'_> { self.as_path().into() }
}

impl AsPathDyn for PathDyn<'_> {
	fn as_path_dyn(&self) -> PathDyn<'_> { *self }
}

impl AsPathDyn for PathBufDyn {
	fn as_path_dyn(&self) -> PathDyn<'_> { self.borrow() }
}

impl AsPathDyn for &PathBufDyn {
	fn as_path_dyn(&self) -> PathDyn<'_> { self.borrow() }
}

impl AsPathDyn for PathCow<'_> {
	fn as_path_dyn(&self) -> PathDyn<'_> {
		match self {
			Self::Borrowed(p) => p.as_path_dyn(),
			Self::Owned(p) => p.as_path_dyn(),
		}
	}
}

// --- AsPathRef
pub trait AsPathRef<'a> {
	fn as_path_ref(self) -> PathDyn<'a>;
}

impl<'a> AsPathRef<'a> for PathDyn<'a> {
	fn as_path_ref(self) -> PathDyn<'a> { self }
}
