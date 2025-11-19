use std::ffi::OsString;

use serde::Serialize;

use crate::{path::PathDyn, scheme::SchemeKind, strand::{AsStrand, Strand, StrandError, StrandKind}};

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
	fn eq(&self, other: &Strand<'_>) -> bool { self.as_strand() == *other }
}

impl StrandBuf {
	pub fn clear(&mut self) {
		match self {
			Self::Os(buf) => buf.clear(),
			Self::Utf8(buf) => buf.clear(),
			Self::Bytes(buf) => buf.clear(),
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

	pub fn into_encoded_bytes(self) -> Vec<u8> {
		match self {
			Self::Os(s) => s.into_encoded_bytes(),
			Self::Utf8(s) => s.into_bytes(),
			Self::Bytes(b) => b,
		}
	}

	pub fn try_push<T>(&mut self, s: T) -> Result<(), StrandError>
	where
		T: AsStrand,
	{
		let s = s.as_strand();
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
}
