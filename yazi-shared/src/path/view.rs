// --- AsPathView
pub trait AsPathView<'a, T> {
	fn as_path_view(self) -> T;
}

impl<'a> AsPathView<'a, &'a std::path::Path> for &'a std::path::Path {
	fn as_path_view(self) -> &'a std::path::Path { self }
}

impl<'a> AsPathView<'a, &'a std::path::Path> for &'a std::path::PathBuf {
	fn as_path_view(self) -> &'a std::path::Path { self }
}

impl<'a> AsPathView<'a, &'a typed_path::UnixPath> for &'a typed_path::UnixPath {
	fn as_path_view(self) -> &'a typed_path::UnixPath { self }
}

impl<'a> AsPathView<'a, &'a typed_path::UnixPath> for &'a typed_path::UnixPathBuf {
	fn as_path_view(self) -> &'a typed_path::UnixPath { self }
}
