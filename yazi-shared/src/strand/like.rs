use std::{borrow::Cow, ffi::OsStr, fmt::Display};

use crate::strand::{AsStrand, StrandBuf, StrandCow, StrandError, StrandKind};

// --- StrandLike
pub trait StrandLike: AsStrand {
	fn as_os(&self) -> Result<&OsStr, StrandError> { self.as_strand().as_os() }

	fn as_utf8(&self) -> Result<&str, StrandError> { self.as_strand().as_utf8() }

	#[cfg(windows)]
	fn backslash_to_slash(&self) -> StrandCow<'_> { self.as_strand().backslash_to_slash() }

	fn contains(&self, x: impl AsStrand) -> bool { self.as_strand().contains(x) }

	fn display(&self) -> impl Display { self.as_strand().display() }

	fn encoded_bytes(&self) -> &[u8] { self.as_strand().encoded_bytes() }

	fn eq_ignore_ascii_case(&self, other: impl AsStrand) -> bool {
		self.as_strand().eq_ignore_ascii_case(other)
	}

	fn is_empty(&self) -> bool { self.as_strand().is_empty() }

	fn kind(&self) -> StrandKind { self.as_strand().kind() }

	fn len(&self) -> usize { self.as_strand().len() }

	fn starts_with(&self, needle: impl AsStrand) -> bool { self.as_strand().starts_with(needle) }

	fn to_owned(&self) -> StrandBuf { self.as_strand().to_owned() }

	fn to_str(&self) -> Result<&str, std::str::Utf8Error> { self.as_strand().to_str() }

	fn to_string_lossy(&self) -> Cow<'_, str> { self.as_strand().to_string_lossy() }
}

impl<S> From<&S> for StrandBuf
where
	S: StrandLike,
{
	fn from(value: &S) -> Self { value.to_owned() }
}

impl StrandLike for StrandBuf {}
impl StrandLike for StrandCow<'_> {}
