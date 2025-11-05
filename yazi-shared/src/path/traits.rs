use crate::path::PathLike;

pub trait AsPath {
	fn as_path(&self) -> impl PathLike<'_>;
}

impl AsPath for &std::ffi::OsStr {
	fn as_path(&self) -> impl PathLike<'_> { std::path::Path::new(self) }
}

impl AsPath for &std::path::Path {
	fn as_path(&self) -> impl PathLike<'_> { *self }
}

impl AsPath for std::path::PathBuf {
	fn as_path(&self) -> impl PathLike<'_> { self.as_path() }
}

impl AsPath for &std::path::PathBuf {
	fn as_path(&self) -> impl PathLike<'_> { (*self).as_path() }
}

// --- AsPathView
pub trait AsPathView<'a, T> {
	fn as_path_view(self) -> T;
}

impl<'a> AsPathView<'a, &'a std::path::Path> for &'a str {
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

impl<'a> AsPathView<'a, super::PathDyn<'a>> for &'a super::PathBufDyn {
	fn as_path_view(self) -> super::PathDyn<'a> {
		match self {
			super::PathBufDyn::Os(p) => super::PathDyn::Os(p.as_path()),
		}
	}
}

impl<'a> AsPathView<'a, &'a std::path::Path> for crate::loc::Loc<'a, &'a std::path::Path> {
	fn as_path_view(self) -> &'a std::path::Path { *self }
}

// --- ToPathOwned
pub trait ToPathOwned<T> {
	fn to_path_owned(&self) -> T;
}

impl ToPathOwned<std::path::PathBuf> for std::path::Path {
	fn to_path_owned(&self) -> std::path::PathBuf { self.to_owned() }
}

impl<T, U> ToPathOwned<U> for &T
where
	T: ?Sized + ToPathOwned<U>,
{
	fn to_path_owned(&self) -> U { (*self).to_path_owned() }
}

// --- AsInnerView
pub trait AsInnerView<'a, T> {
	fn as_inner_view(&'a self) -> T;
}

impl<'a> AsInnerView<'a, &'a std::ffi::OsStr> for std::ffi::OsStr {
	fn as_inner_view(&'a self) -> &'a std::ffi::OsStr { self }
}

impl<'a> AsInnerView<'a, &'a std::ffi::OsStr> for std::ffi::OsString {
	fn as_inner_view(&'a self) -> &'a std::ffi::OsStr { self }
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
