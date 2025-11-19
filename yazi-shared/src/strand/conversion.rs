use std::{borrow::Cow, ffi::{OsStr, OsString}};

use crate::{path::{PathBufDyn, PathDyn}, strand::{Strand, StrandBuf, StrandCow}};

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

impl AsStrand for Vec<u8> {
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

impl AsStrand for Cow<'_, OsStr> {
	fn as_strand(&self) -> Strand<'_> { Strand::Os(self) }
}

impl AsStrand for PathDyn<'_> {
	fn as_strand(&self) -> Strand<'_> {
		match self {
			Self::Os(p) => Strand::Os(p.as_os_str()),
		}
	}
}

impl AsStrand for &PathBufDyn {
	fn as_strand(&self) -> Strand<'_> {
		match self {
			PathBufDyn::Os(p) => Strand::Os(p.as_os_str()),
		}
	}
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
