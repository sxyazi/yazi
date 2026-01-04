use std::{borrow::Cow, ffi::OsStr, fmt::Display};

use anyhow::Result;

use crate::{BytesExt, strand::{AsStrand, StrandBuf, StrandError, StrandKind}, wtf8::FromWtf8};

// --- Strand
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialOrd)]
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

impl<'a> From<&'a [u8]> for Strand<'a> {
	fn from(value: &'a [u8]) -> Self { Self::Bytes(value) }
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

impl PartialEq for Strand<'_> {
	fn eq(&self, other: &Self) -> bool {
		match *other {
			Self::Os(s) => *self == s,
			Self::Utf8(s) => *self == s,
			Self::Bytes(b) => *self == b,
		}
	}
}

impl PartialEq<&OsStr> for Strand<'_> {
	fn eq(&self, other: &&OsStr) -> bool {
		match *self {
			Self::Os(s) => s == *other,
			Self::Utf8(s) => s == *other,
			Self::Bytes(b) => b == other.as_encoded_bytes(),
		}
	}
}

impl PartialEq<&str> for Strand<'_> {
	fn eq(&self, other: &&str) -> bool {
		match *self {
			Self::Os(s) => s == *other,
			Self::Utf8(s) => s == *other,
			Self::Bytes(b) => b == other.as_bytes(),
		}
	}
}

impl PartialEq<&[u8]> for Strand<'_> {
	fn eq(&self, other: &&[u8]) -> bool {
		match *self {
			Self::Os(s) => s.as_encoded_bytes() == *other,
			Self::Utf8(s) => s.as_bytes() == *other,
			Self::Bytes(b) => b == *other,
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
		out.reserve_exact(bytes.len());
		out.extend(skip);

		for &b in rest {
			out.push(if b == b'\\' { b'/' } else { b });
		}
		unsafe { StrandBuf::from_encoded_bytes(self.kind(), out) }.into()
	}

	pub fn contains(self, x: impl AsStrand) -> bool {
		memchr::memmem::find(self.encoded_bytes(), x.as_strand().encoded_bytes()).is_some()
	}

	pub fn display(self) -> impl Display {
		struct D<'a>(Strand<'a>);

		impl<'a> Display for D<'a> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self.0 {
					Strand::Os(s) => s.display().fmt(f),
					Strand::Utf8(s) => s.fmt(f),
					Strand::Bytes(b) => b.display().fmt(f),
				}
			}
		}

		D(self)
	}

	#[inline]
	pub fn encoded_bytes(self) -> &'a [u8] {
		match self {
			Self::Os(s) => s.as_encoded_bytes(),
			Self::Utf8(s) => s.as_bytes(),
			Self::Bytes(b) => b,
		}
	}

	pub fn eq_ignore_ascii_case(self, other: impl AsStrand) -> bool {
		self.encoded_bytes().eq_ignore_ascii_case(other.as_strand().encoded_bytes())
	}

	#[inline]
	pub unsafe fn from_encoded_bytes(kind: impl Into<StrandKind>, bytes: &'a [u8]) -> Self {
		match kind.into() {
			StrandKind::Utf8 => Self::Utf8(unsafe { str::from_utf8_unchecked(bytes) }),
			StrandKind::Os => Self::Os(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) }),
			StrandKind::Bytes => Self::Bytes(bytes),
		}
	}

	pub fn is_empty(self) -> bool { self.encoded_bytes().is_empty() }

	pub fn kind(self) -> StrandKind {
		match self {
			Self::Utf8(_) => StrandKind::Utf8,
			Self::Os(_) => StrandKind::Os,
			Self::Bytes(_) => StrandKind::Bytes,
		}
	}

	pub fn len(self) -> usize { self.encoded_bytes().len() }

	pub fn starts_with(self, needle: impl AsStrand) -> bool {
		self.encoded_bytes().starts_with(needle.as_strand().encoded_bytes())
	}

	pub fn to_owned(self) -> StrandBuf {
		match self {
			Self::Os(s) => StrandBuf::Os(s.to_owned()),
			Self::Utf8(s) => StrandBuf::Utf8(s.to_owned()),
			Self::Bytes(b) => StrandBuf::Bytes(b.to_owned()),
		}
	}

	pub fn to_str(self) -> Result<&'a str, std::str::Utf8Error> {
		str::from_utf8(self.encoded_bytes())
	}

	pub fn to_string_lossy(self) -> Cow<'a, str> { String::from_utf8_lossy(self.encoded_bytes()) }

	pub fn with<K, S>(kind: K, strand: &'a S) -> Result<Self>
	where
		K: Into<StrandKind>,
		S: ?Sized + AsStrand,
	{
		let strand = strand.as_strand();
		Ok(match kind.into() {
			StrandKind::Utf8 => Self::Utf8(strand.as_utf8()?),
			StrandKind::Os => Self::Os(strand.as_os()?),
			StrandKind::Bytes => Self::Bytes(strand.encoded_bytes()),
		})
	}
}
