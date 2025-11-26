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

impl<'a> AsStrandView<'a, &'a [u8]> for &'a [u8] {
	fn as_strand_view(self) -> &'a [u8] { self }
}

impl<'a> AsStrandView<'a, &'a [u8]> for &'a typed_path::UnixPath {
	fn as_strand_view(self) -> &'a [u8] { self.as_bytes() }
}

impl<'a> AsStrandView<'a, &'a [u8]> for typed_path::UnixComponents<'a> {
	fn as_strand_view(self) -> &'a [u8] { self.as_path::<typed_path::UnixEncoding>().as_bytes() }
}
