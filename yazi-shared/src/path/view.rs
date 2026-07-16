// --- PathView
pub trait PathView<'a, T> {
	fn path_view(self) -> T;
}

impl<'a> PathView<'a, &'a std::path::Path> for &'a std::path::Path {
	fn path_view(self) -> &'a std::path::Path { self }
}

impl<'a> PathView<'a, &'a std::path::Path> for &'a std::path::PathBuf {
	fn path_view(self) -> &'a std::path::Path { self }
}

impl<'a> PathView<'a, &'a typed_path::UnixPath> for &'a typed_path::UnixPath {
	fn path_view(self) -> &'a typed_path::UnixPath { self }
}

impl<'a> PathView<'a, &'a typed_path::UnixPath> for &'a typed_path::UnixPathBuf {
	fn path_view(self) -> &'a typed_path::UnixPath { self }
}
