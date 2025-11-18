use std::{borrow::Cow, ffi::{OsStr, OsString}};

use crate::{path::{PathBufDyn, PathDyn}, strand::{Strand, StrandBuf, StrandBufLike, StrandCow, StrandLike}};

// --- AsStrand
pub trait AsStrand {
	fn as_strand(&self) -> impl StrandLike<'_>;
}

impl AsStrand for [u8] {
	fn as_strand(&self) -> impl StrandLike<'_> { self }
}

impl AsStrand for &[u8] {
	fn as_strand(&self) -> impl StrandLike<'_> { *self }
}

impl AsStrand for Vec<u8> {
	fn as_strand(&self) -> impl StrandLike<'_> { self.as_slice() }
}

impl AsStrand for str {
	fn as_strand(&self) -> impl StrandLike<'_> { self }
}

impl AsStrand for &str {
	fn as_strand(&self) -> impl StrandLike<'_> { *self }
}

impl AsStrand for String {
	fn as_strand(&self) -> impl StrandLike<'_> { self.as_str() }
}

impl AsStrand for OsStr {
	fn as_strand(&self) -> impl StrandLike<'_> { self }
}

impl AsStrand for &OsStr {
	fn as_strand(&self) -> impl StrandLike<'_> { *self }
}

impl AsStrand for OsString {
	fn as_strand(&self) -> impl StrandLike<'_> { self.as_os_str() }
}

impl AsStrand for Cow<'_, OsStr> {
	fn as_strand(&self) -> impl StrandLike<'_> { AsRef::<OsStr>::as_ref(self) }
}

impl AsStrand for PathDyn<'_> {
	fn as_strand(&self) -> impl StrandLike<'_> {
		match self {
			Self::Os(p) => p.as_os_str(),
		}
	}
}

impl AsStrand for &PathBufDyn {
	fn as_strand(&self) -> impl StrandLike<'_> {
		match self {
			PathBufDyn::Os(p) => p.as_os_str(),
		}
	}
}

impl AsStrand for Strand<'_> {
	fn as_strand(&self) -> impl StrandLike<'_> { *self }
}

impl AsStrand for StrandBuf {
	fn as_strand(&self) -> impl StrandLike<'_> { self.borrow() }
}

impl AsStrand for StrandCow<'_> {
	fn as_strand(&self) -> impl StrandLike<'_> {
		match self {
			StrandCow::Borrowed(s) => *s,
			StrandCow::Owned(s) => s.into(),
		}
	}
}

impl AsStrand for &StrandCow<'_> {
	fn as_strand(&self) -> impl StrandLike<'_> { (**self).as_strand() }
}

// --- AsStrandDyn
pub trait AsStrandDyn {
	fn as_strand_dyn(&self) -> Strand<'_>;
}

impl AsStrandDyn for OsStr {
	fn as_strand_dyn(&self) -> Strand<'_> { Strand::Os(self) }
}

impl AsStrandDyn for &OsStr {
	fn as_strand_dyn(&self) -> Strand<'_> { Strand::Os(self) }
}

impl AsStrandDyn for OsString {
	fn as_strand_dyn(&self) -> Strand<'_> { Strand::Os(self) }
}

impl AsStrandDyn for &std::path::PathBuf {
	fn as_strand_dyn(&self) -> Strand<'_> { Strand::Os(self.as_os_str()) }
}

impl AsStrandDyn for &str {
	fn as_strand_dyn(&self) -> Strand<'_> { Strand::Utf8(self) }
}

impl AsStrandDyn for String {
	fn as_strand_dyn(&self) -> Strand<'_> { Strand::Utf8(self) }
}

impl AsStrandDyn for &String {
	fn as_strand_dyn(&self) -> Strand<'_> { Strand::Utf8(self) }
}

impl AsStrandDyn for Cow<'_, OsStr> {
	fn as_strand_dyn(&self) -> Strand<'_> { Strand::Os(self) }
}

impl AsStrandDyn for PathDyn<'_> {
	fn as_strand_dyn(&self) -> Strand<'_> {
		match self {
			Self::Os(p) => Strand::Os(p.as_os_str()),
		}
	}
}

impl AsStrandDyn for &PathBufDyn {
	fn as_strand_dyn(&self) -> Strand<'_> {
		match self {
			PathBufDyn::Os(p) => Strand::Os(p.as_os_str()),
		}
	}
}

impl AsStrandDyn for Strand<'_> {
	fn as_strand_dyn(&self) -> Strand<'_> { *self }
}

impl AsStrandDyn for StrandBuf {
	fn as_strand_dyn(&self) -> Strand<'_> { self.borrow() }
}

impl AsStrandDyn for &StrandBuf {
	fn as_strand_dyn(&self) -> Strand<'_> { (*self).borrow() }
}

impl AsStrandDyn for StrandCow<'_> {
	fn as_strand_dyn(&self) -> Strand<'_> {
		match self {
			Self::Borrowed(s) => s.as_strand_dyn(),
			Self::Owned(s) => s.as_strand_dyn(),
		}
	}
}
