use std::ffi::OsStr;

// --- AsStrandView
pub trait AsStrandView<'a, T> {
	fn as_strand_view(self) -> T;
}

impl<'a> AsStrandView<'a, &'a OsStr> for &'a OsStr {
	fn as_strand_view(self) -> &'a OsStr { self }
}

impl<'a> AsStrandView<'a, &'a OsStr> for &'a std::path::Path {
	fn as_strand_view(self) -> &'a OsStr { self.as_os_str() }
}

impl<'a> AsStrandView<'a, &'a OsStr> for std::path::Components<'a> {
	fn as_strand_view(self) -> &'a OsStr { self.as_path().as_os_str() }
}
