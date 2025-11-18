use std::{borrow::Cow, ffi::{OsStr, OsString}, fmt::Display};

use crate::{BytesExt, strand::{AsStrand, Strand, StrandBuf, StrandKind}};

// --- StrandLike
pub trait StrandLike<'a>: Copy {
	type Owned: StrandBufLike;

	fn contains(self, x: impl AsStrand) -> bool {
		memchr::memmem::find(self.encoded_bytes(), x.as_strand().encoded_bytes()).is_some()
	}

	fn display(self) -> impl Display;

	fn encoded_bytes(self) -> &'a [u8];

	fn is_empty(self) -> bool { self.encoded_bytes().is_empty() }

	fn kind(self) -> StrandKind;

	fn len(self) -> usize { self.encoded_bytes().len() }

	fn to_str(self) -> Result<&'a str, std::str::Utf8Error> { str::from_utf8(self.encoded_bytes()) }

	fn to_string_lossy(self) -> Cow<'a, str> { String::from_utf8_lossy(self.encoded_bytes()) }

	fn eq_ignore_ascii_case(self, other: impl AsStrand) -> bool {
		self.encoded_bytes().eq_ignore_ascii_case(other.as_strand().encoded_bytes())
	}

	fn starts_with(self, needle: impl AsStrand) -> bool {
		self.encoded_bytes().starts_with(needle.as_strand().encoded_bytes())
	}
}

impl<'a> StrandLike<'a> for &'a [u8] {
	type Owned = Vec<u8>;

	fn display(self) -> impl Display { BytesExt::display(self) }

	fn encoded_bytes(self) -> &'a [u8] { self }

	fn kind(self) -> StrandKind { StrandKind::Bytes }
}

impl<'a> StrandLike<'a> for &'a str {
	type Owned = String;

	fn display(self) -> impl Display { self }

	fn encoded_bytes(self) -> &'a [u8] { self.as_bytes() }

	fn kind(self) -> StrandKind { StrandKind::Utf8 }
}

impl<'a> StrandLike<'a> for &'a OsStr {
	type Owned = OsString;

	fn display(self) -> impl Display { self.display() }

	fn encoded_bytes(self) -> &'a [u8] { self.as_encoded_bytes() }

	fn kind(self) -> StrandKind { StrandKind::Os }
}

impl<'a> StrandLike<'a> for Strand<'a> {
	type Owned = StrandBuf;

	fn display(self) -> impl Display {
		struct D<'a>(Strand<'a>);

		impl<'a> Display for D<'a> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self.0 {
					Strand::Os(s) => s.display().fmt(f),
					Strand::Utf8(s) => s.fmt(f),
					Strand::Bytes(b) => BytesExt::display(b).fmt(f),
				}
			}
		}

		D(self)
	}

	fn encoded_bytes(self) -> &'a [u8] {
		match self {
			Self::Os(s) => s.as_encoded_bytes(),
			Self::Utf8(s) => s.as_bytes(),
			Self::Bytes(b) => b,
		}
	}

	fn kind(self) -> StrandKind {
		match self {
			Self::Os(_) => StrandKind::Os,
			Self::Utf8(_) => StrandKind::Utf8,
			Self::Bytes(_) => StrandKind::Bytes,
		}
	}
}

// --- StrandBufLike
pub trait StrandBufLike
where
	Self: 'static + AsStrand,
{
	type Borrowed<'a>: StrandLike<'a>;

	fn borrow(&self) -> Self::Borrowed<'_>;

	fn encoded_bytes(&self) -> &[u8] { self.borrow().encoded_bytes() }

	fn into_encoded_bytes(self) -> Vec<u8>;

	fn is_empty(&self) -> bool { self.borrow().is_empty() }

	fn kind(&self) -> StrandKind { self.borrow().kind() }

	fn len(&self) -> usize { self.borrow().len() }

	fn to_str(&self) -> Result<&str, std::str::Utf8Error> { self.borrow().to_str() }

	fn to_string_lossy(&self) -> Cow<'_, str> { self.borrow().to_string_lossy() }
}

impl StrandBufLike for Vec<u8> {
	type Borrowed<'a> = &'a [u8];

	fn borrow(&self) -> Self::Borrowed<'_> { self.as_slice() }

	fn into_encoded_bytes(self) -> Vec<u8> { self }
}

impl StrandBufLike for String {
	type Borrowed<'a> = &'a str;

	fn borrow(&self) -> Self::Borrowed<'_> { self.as_str() }

	fn into_encoded_bytes(self) -> Vec<u8> { self.into_bytes() }
}

impl StrandBufLike for OsString {
	type Borrowed<'a> = &'a OsStr;

	fn borrow(&self) -> Self::Borrowed<'_> { self.as_os_str() }

	fn into_encoded_bytes(self) -> Vec<u8> { self.into_encoded_bytes() }
}

impl StrandBufLike for StrandBuf {
	type Borrowed<'a> = Strand<'a>;

	fn borrow(&self) -> Self::Borrowed<'_> {
		match self {
			Self::Os(s) => Strand::Os(s.as_os_str()),
			Self::Utf8(s) => Strand::Utf8(s.as_str()),
			Self::Bytes(b) => Strand::Bytes(b),
		}
	}

	fn into_encoded_bytes(self) -> Vec<u8> {
		match self {
			Self::Os(s) => s.into_encoded_bytes(),
			Self::Utf8(s) => s.into_bytes(),
			Self::Bytes(b) => b,
		}
	}
}
