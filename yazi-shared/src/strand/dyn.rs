use std::ffi::{OsStr, OsString};

use serde::Serialize;

use crate::{FromWtf8, path::PathDyn, scheme::SchemeKind, strand::{AsStrandDyn, StrandBufLike, StrandError, StrandKind, StrandLike}};

// --- Strand
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Strand<'p> {
	Os(&'p OsStr),
	Utf8(&'p str),
	Bytes(&'p [u8]),
}

impl Default for Strand<'_> {
	fn default() -> Self { Self::Utf8("") }
}

impl<'a> From<&'a OsStr> for Strand<'a> {
	fn from(value: &'a OsStr) -> Self { Self::Os(value) }
}

impl<'a> From<&'a str> for Strand<'a> {
	fn from(value: &'a str) -> Self { Self::Utf8(value) }
}

impl<'a> From<&'a StrandBuf> for Strand<'a> {
	fn from(value: &'a StrandBuf) -> Self {
		match value {
			StrandBuf::Os(s) => Self::Os(s),
			StrandBuf::Utf8(s) => Self::Utf8(s),
			StrandBuf::Bytes(s) => Self::Bytes(s),
		}
	}
}

impl PartialEq<&str> for Strand<'_> {
	fn eq(&self, other: &&str) -> bool {
		match self {
			Self::Os(s) => s == other,
			Self::Utf8(s) => s == other,
			Self::Bytes(b) => *b == other.as_bytes(),
		}
	}
}

impl<'a> Strand<'a> {
	pub fn to_owned(self) -> StrandBuf {
		match self {
			Self::Os(s) => StrandBuf::Os(s.to_owned()),
			Self::Utf8(s) => StrandBuf::Utf8(s.to_owned()),
			Self::Bytes(b) => StrandBuf::Bytes(b.to_owned()),
		}
	}
}

impl<'a> Strand<'a> {
	#[inline]
	pub fn as_os(self) -> Result<&'a OsStr, StrandError> {
		match self {
			Self::Os(s) => Ok(s),
			Self::Utf8(s) => Ok(OsStr::new(s)),
			Self::Bytes(b) => OsStr::from_wtf8(b).map_err(|_| StrandError::AsOs),
		}
	}

	#[inline]
	pub fn as_utf8(self) -> Result<&'a str, StrandError> {
		match self {
			Self::Os(s) => s.to_str().ok_or(StrandError::AsUtf8),
			Self::Utf8(s) => Ok(s),
			Self::Bytes(b) => str::from_utf8(b).map_err(|_| StrandError::AsUtf8),
		}
	}

	#[cfg(windows)]
	pub fn backslash_to_slash(self) -> super::StrandCow<'a> {
		let bytes = self.encoded_bytes();

		// Fast path to skip if there are no backslashes
		let skip_len = bytes.iter().take_while(|&&b| b != b'\\').count();
		if skip_len >= bytes.len() {
			return self.into();
		}

		let (skip, rest) = bytes.split_at(skip_len);
		let mut out = Vec::new();
		out.try_reserve_exact(bytes.len()).unwrap_or_else(|_| panic!());
		out.extend(skip);

		for &b in rest {
			out.push(if b == b'\\' { b'/' } else { b });
		}
		unsafe { StrandBuf::from_encoded_bytes(self.kind(), out) }.into()
	}

	#[inline]
	pub unsafe fn from_encoded_bytes(kind: impl Into<StrandKind>, bytes: &'a [u8]) -> Self {
		match kind.into() {
			StrandKind::Os => Self::Os(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) }),
			StrandKind::Utf8 => Self::Utf8(unsafe { str::from_utf8_unchecked(bytes) }),
			StrandKind::Bytes => Self::Bytes(bytes),
		}
	}
}

// --- StrandBuf
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
pub enum StrandBuf {
	Os(OsString),
	Utf8(String),
	Bytes(Vec<u8>),
}

impl Default for StrandBuf {
	fn default() -> Self { Self::Utf8(String::new()) }
}

impl From<OsString> for StrandBuf {
	fn from(value: OsString) -> Self { Self::Os(value) }
}

impl From<String> for StrandBuf {
	fn from(value: String) -> Self { Self::Utf8(value) }
}

impl From<PathDyn<'_>> for StrandBuf {
	fn from(value: PathDyn) -> Self {
		match value {
			PathDyn::Os(p) => Self::Os(p.as_os_str().to_owned()),
		}
	}
}

impl PartialEq<Strand<'_>> for StrandBuf {
	fn eq(&self, other: &Strand<'_>) -> bool { self.borrow() == *other }
}

impl StrandBuf {
	pub fn clear(&mut self) {
		match self {
			Self::Os(buf) => buf.clear(),
			Self::Utf8(buf) => buf.clear(),
			Self::Bytes(buf) => buf.clear(),
		}
	}

	pub fn try_push<T>(&mut self, s: T) -> Result<(), StrandError>
	where
		T: AsStrandDyn,
	{
		let s = s.as_strand_dyn();
		Ok(match self {
			Self::Os(buf) => buf.push(s.as_os()?),
			Self::Utf8(buf) => buf.push_str(s.as_utf8()?),
			Self::Bytes(buf) => buf.extend(s.encoded_bytes()),
		})
	}

	pub fn with_capacity(kind: SchemeKind, capacity: usize) -> Self {
		use SchemeKind as K;
		match kind {
			K::Regular | K::Search | K::Archive => Self::Os(OsString::with_capacity(capacity)),
			K::Sftp => Self::Os(OsString::with_capacity(capacity)), // FIXME
		}
	}

	#[inline]
	pub unsafe fn from_encoded_bytes(kind: impl Into<StrandKind>, bytes: Vec<u8>) -> Self {
		match kind.into() {
			StrandKind::Os => Self::Os(unsafe { OsString::from_encoded_bytes_unchecked(bytes) }),
			StrandKind::Utf8 => Self::Utf8(unsafe { String::from_utf8_unchecked(bytes) }),
			StrandKind::Bytes => Self::Bytes(bytes),
		}
	}
}
