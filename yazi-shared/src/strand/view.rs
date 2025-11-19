use std::ffi::OsStr;

use crate::{path::PathDyn, strand::Strand};

// --- AsStrandView
pub trait AsStrandView<'a, T> {
	fn as_strand_view(self) -> T;
}

impl<'a> AsStrandView<'a, &'a OsStr> for &'a str {
	fn as_strand_view(self) -> &'a OsStr { OsStr::new(self) }
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

impl<'a> AsStrandView<'a, Strand<'a>> for &'a str {
	fn as_strand_view(self) -> Strand<'a> { Strand::Utf8(self) }
}

impl<'a> AsStrandView<'a, Strand<'a>> for &'a std::path::Path {
	fn as_strand_view(self) -> Strand<'a> { Strand::Os(self.as_os_str()) }
}

impl<'a> AsStrandView<'a, Strand<'a>> for std::path::Components<'a> {
	fn as_strand_view(self) -> Strand<'a> { Strand::Os(self.as_path().as_os_str()) }
}

impl<'a> AsStrandView<'a, Strand<'a>> for PathDyn<'a> {
	fn as_strand_view(self) -> Strand<'a> {
		match self {
			Self::Os(p) => Strand::Os(p.as_os_str()),
		}
	}
}
