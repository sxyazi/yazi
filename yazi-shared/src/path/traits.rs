use std::{borrow::Cow, ffi::{OsStr, OsString}};

use super::{PathBufDyn, PathDyn};
use crate::path::{PathBufLike, PathLike};

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

// --- AsPathDyn
pub trait AsPathDyn {
	fn as_path_dyn(&self) -> PathDyn<'_>;
}

impl AsPathDyn for &str {
	fn as_path_dyn(&self) -> PathDyn<'_> { std::path::Path::new(self).into() }
}

impl AsPathDyn for String {
	fn as_path_dyn(&self) -> PathDyn<'_> { std::path::Path::new(self).into() }
}

impl AsPathDyn for &String {
	fn as_path_dyn(&self) -> PathDyn<'_> { std::path::Path::new(self).into() }
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

// --- AsPathView
pub trait AsPathView<'a, T> {
	fn as_path_view(self) -> T;
}

impl<'a> AsPathView<'a, &'a std::path::Path> for &'a str {
	fn as_path_view(self) -> &'a std::path::Path { std::path::Path::new(self) }
}

impl<'a> AsPathView<'a, &'a std::path::Path> for &'a OsStr {
	fn as_path_view(self) -> &'a std::path::Path { std::path::Path::new(self) }
}

impl<'a> AsPathView<'a, &'a std::path::Path> for &'a std::path::Path {
	fn as_path_view(self) -> &'a std::path::Path { self }
}

impl<'a> AsPathView<'a, &'a std::path::Path> for &'a std::path::PathBuf {
	fn as_path_view(self) -> &'a std::path::Path { self }
}

impl<'a> AsPathView<'a, &'a std::path::Path> for std::path::Components<'a> {
	fn as_path_view(self) -> &'a std::path::Path { self.as_path() }
}

impl<'a> AsPathView<'a, PathDyn<'a>> for &'a PathBufDyn {
	fn as_path_view(self) -> PathDyn<'a> {
		match self {
			PathBufDyn::Os(p) => PathDyn::Os(p.as_path()),
		}
	}
}

impl<'a> AsPathView<'a, &'a std::path::Path> for &'a Cow<'_, OsStr> {
	fn as_path_view(self) -> &'a std::path::Path { std::path::Path::new(self) }
}

impl<'a> AsPathView<'a, &'a std::path::Path> for crate::loc::Loc<'a, &'a std::path::Path> {
	fn as_path_view(self) -> &'a std::path::Path { *self }
}

// --- AsInnerView
pub trait AsInnerView<'a, T> {
	fn as_inner_view(&'a self) -> T;
}

impl<'a> AsInnerView<'a, &'a OsStr> for OsStr {
	fn as_inner_view(&'a self) -> &'a OsStr { self }
}

impl<'a> AsInnerView<'a, &'a OsStr> for OsString {
	fn as_inner_view(&'a self) -> &'a OsStr { self }
}

impl<'a> AsInnerView<'a, &'a [u8]> for [u8] {
	fn as_inner_view(&'a self) -> &'a [u8] { self }
}

impl<'a> AsInnerView<'a, &'a [u8]> for Vec<u8> {
	fn as_inner_view(&'a self) -> &'a [u8] { self }
}

impl<'a, T, U> AsInnerView<'a, U> for &T
where
	T: ?Sized + AsInnerView<'a, U>,
{
	fn as_inner_view(&'a self) -> U { (*self).as_inner_view() }
}
