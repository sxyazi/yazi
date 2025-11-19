use std::{borrow::Cow, ffi::OsStr};

use crate::path::{AsPath, PathDyn};

// --- AsPathView
pub trait AsPathView<'a, T> {
	fn as_path_view(self) -> T;
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

impl<'a, T> AsPathView<'a, PathDyn<'a>> for &'a T
where
	T: AsPath,
{
	fn as_path_view(self) -> PathDyn<'a> { self.as_path() }
}

impl<'a> AsPathView<'a, &'a std::path::Path> for &'a Cow<'_, OsStr> {
	fn as_path_view(self) -> &'a std::path::Path { std::path::Path::new(self) }
}

impl<'a> AsPathView<'a, &'a std::path::Path> for crate::loc::Loc<'a, &'a std::path::Path> {
	fn as_path_view(self) -> &'a std::path::Path { *self }
}
