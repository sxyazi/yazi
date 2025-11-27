use std::{borrow::Cow, ffi::OsString, hash::{Hash, Hasher}};

use anyhow::Result;

use crate::{path::PathDyn, strand::{AsStrand, Strand, StrandCow, StrandError, StrandKind}, wtf8::FromWtf8Vec};

// --- StrandBuf
#[derive(Clone, Debug, Eq)]
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

impl From<&str> for StrandBuf {
	fn from(value: &str) -> Self { Self::Utf8(value.to_owned()) }
}

impl From<String> for StrandBuf {
	fn from(value: String) -> Self { Self::Utf8(value) }
}

impl From<PathDyn<'_>> for StrandBuf {
	fn from(value: PathDyn) -> Self {
		match value {
			PathDyn::Os(p) => Self::Os(p.as_os_str().to_owned()),
			PathDyn::Unix(p) => Self::Bytes(p.as_bytes().to_owned()),
		}
	}
}

impl From<StrandCow<'_>> for StrandBuf {
	fn from(value: StrandCow<'_>) -> Self { value.into_owned() }
}

impl PartialEq for StrandBuf {
	fn eq(&self, other: &Self) -> bool { self.as_strand() == other.as_strand() }
}

impl PartialEq<Strand<'_>> for StrandBuf {
	fn eq(&self, other: &Strand<'_>) -> bool { self.as_strand() == *other }
}

impl Hash for StrandBuf {
	fn hash<H: Hasher>(&self, state: &mut H) { self.as_strand().hash(state); }
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
			StrandKind::Utf8 => Self::Utf8(unsafe { String::from_utf8_unchecked(bytes) }),
			StrandKind::Os => Self::Os(unsafe { OsString::from_encoded_bytes_unchecked(bytes) }),
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

	pub fn into_string_lossy(self) -> String {
		match self {
			Self::Os(s) => match s.to_string_lossy() {
				Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(s.into_encoded_bytes()) },
				Cow::Owned(s) => s,
			},
			Self::Utf8(s) => s,
			Self::Bytes(b) => match String::from_utf8_lossy(&b) {
				Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(b) },
				Cow::Owned(s) => s,
			},
		}
	}

	pub fn new(kind: impl Into<StrandKind>) -> Self { Self::with_str(kind, "") }

	pub fn push_str(&mut self, s: impl AsRef<str>) {
		let s = s.as_ref();
		match self {
			Self::Os(buf) => buf.push(s),
			Self::Utf8(buf) => buf.push_str(s),
			Self::Bytes(buf) => buf.extend(s.as_bytes()),
		}
	}

	pub fn reserve_exact(&mut self, additional: usize) {
		match self {
			Self::Os(buf) => buf.reserve_exact(additional),
			Self::Utf8(buf) => buf.reserve_exact(additional),
			Self::Bytes(buf) => buf.reserve_exact(additional),
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

	pub fn with<K>(kind: K, bytes: Vec<u8>) -> Result<Self>
	where
		K: Into<StrandKind>,
	{
		Ok(match kind.into() {
			StrandKind::Utf8 => Self::Utf8(String::from_utf8(bytes)?),
			StrandKind::Os => Self::Os(OsString::from_wtf8_vec(bytes)?),
			StrandKind::Bytes => Self::Bytes(bytes),
		})
	}

	pub fn with_capacity<K>(kind: K, capacity: usize) -> Self
	where
		K: Into<StrandKind>,
	{
		match kind.into() {
			StrandKind::Utf8 => Self::Utf8(String::with_capacity(capacity)),
			StrandKind::Os => Self::Os(OsString::with_capacity(capacity)),
			StrandKind::Bytes => Self::Bytes(Vec::with_capacity(capacity)),
		}
	}

	pub fn with_str<K, S>(kind: K, s: S) -> Self
	where
		K: Into<StrandKind>,
		S: Into<String>,
	{
		let s = s.into();
		match kind.into() {
			StrandKind::Utf8 => Self::Utf8(s),
			StrandKind::Os => Self::Os(s.into()),
			StrandKind::Bytes => Self::Bytes(s.into_bytes()),
		}
	}
}
