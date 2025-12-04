use std::{borrow::Cow, ffi::{OsStr, OsString}};

use crate::{path::{PathBufDyn, PathCow, PathDyn}, strand::{Strand, StrandBuf, StrandCow}, url::AsUrl};

// --- AsStrand
pub trait AsStrand {
	fn as_strand(&self) -> Strand<'_>;
}

impl AsStrand for [u8] {
	fn as_strand(&self) -> Strand<'_> { Strand::Bytes(self) }
}

impl AsStrand for &[u8] {
	fn as_strand(&self) -> Strand<'_> { Strand::Bytes(self) }
}

impl AsStrand for str {
	fn as_strand(&self) -> Strand<'_> { Strand::Utf8(self) }
}

impl AsStrand for &str {
	fn as_strand(&self) -> Strand<'_> { Strand::Utf8(self) }
}

impl AsStrand for String {
	fn as_strand(&self) -> Strand<'_> { Strand::Utf8(self) }
}

impl AsStrand for &String {
	fn as_strand(&self) -> Strand<'_> { Strand::Utf8(self) }
}

impl AsStrand for OsStr {
	fn as_strand(&self) -> Strand<'_> { Strand::Os(self) }
}

impl AsStrand for &OsStr {
	fn as_strand(&self) -> Strand<'_> { Strand::Os(self) }
}

impl AsStrand for OsString {
	fn as_strand(&self) -> Strand<'_> { Strand::Os(self) }
}

impl AsStrand for &std::path::Path {
	fn as_strand(&self) -> Strand<'_> { Strand::Os(self.as_os_str()) }
}

impl AsStrand for &std::path::PathBuf {
	fn as_strand(&self) -> Strand<'_> { Strand::Os(self.as_os_str()) }
}

impl AsStrand for &typed_path::UnixPath {
	fn as_strand(&self) -> Strand<'_> { Strand::Bytes(self.as_bytes()) }
}

impl AsStrand for crate::path::Components<'_> {
	fn as_strand(&self) -> Strand<'_> { self.strand() }
}

impl AsStrand for Cow<'_, [u8]> {
	fn as_strand(&self) -> Strand<'_> { Strand::Bytes(self) }
}

impl AsStrand for Cow<'_, OsStr> {
	fn as_strand(&self) -> Strand<'_> { Strand::Os(self) }
}

impl AsStrand for PathDyn<'_> {
	fn as_strand(&self) -> Strand<'_> {
		match self {
			Self::Os(p) => Strand::Os(p.as_os_str()),
			Self::Unix(p) => Strand::Bytes(p.as_bytes()),
		}
	}
}

impl AsStrand for PathBufDyn {
	fn as_strand(&self) -> Strand<'_> {
		match self {
			Self::Os(p) => Strand::Os(p.as_os_str()),
			Self::Unix(p) => Strand::Bytes(p.as_bytes()),
		}
	}
}

impl AsStrand for &PathBufDyn {
	fn as_strand(&self) -> Strand<'_> { (**self).as_strand() }
}

impl AsStrand for PathCow<'_> {
	fn as_strand(&self) -> Strand<'_> {
		match self {
			Self::Borrowed(p) => p.as_strand(),
			Self::Owned(p) => p.as_strand(),
		}
	}
}

impl AsStrand for &PathCow<'_> {
	fn as_strand(&self) -> Strand<'_> { (**self).as_strand() }
}

impl AsStrand for Strand<'_> {
	fn as_strand(&self) -> Strand<'_> { *self }
}

impl AsStrand for StrandBuf {
	fn as_strand(&self) -> Strand<'_> {
		match self {
			Self::Os(s) => Strand::Os(s),
			Self::Utf8(s) => Strand::Utf8(s),
			Self::Bytes(b) => Strand::Bytes(b),
		}
	}
}

impl AsStrand for &StrandBuf {
	fn as_strand(&self) -> Strand<'_> { (**self).as_strand() }
}

impl AsStrand for StrandCow<'_> {
	fn as_strand(&self) -> Strand<'_> {
		match self {
			StrandCow::Borrowed(s) => *s,
			StrandCow::Owned(s) => s.as_strand(),
		}
	}
}

impl AsStrand for &StrandCow<'_> {
	fn as_strand(&self) -> Strand<'_> { (**self).as_strand() }
}

// --- ToStrand
pub trait ToStrand {
	fn to_strand(&self) -> StrandCow<'_>;
}

impl ToStrand for String {
	fn to_strand(&self) -> StrandCow<'_> { self.as_strand().into() }
}

impl<T> ToStrand for T
where
	T: AsUrl,
{
	fn to_strand(&self) -> StrandCow<'_> { self.as_url().components().strand() }
}
