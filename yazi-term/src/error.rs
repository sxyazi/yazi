use std::str;

pub type Result<T, E = ParseError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	/// Malformed or unrecognized byte sequence.
	#[error("invalid or unrecognized byte sequence")]
	Invalid,
	/// Recognised sequence that should not be emitted as an event.
	#[error("recognised sequence that should be ignored")]
	Ignored,
	/// The sequence is not yet complete; more bytes are needed before it can be
	/// parsed.
	#[error("incomplete sequence; more bytes are needed")]
	Incomplete,
}

impl From<str::Utf8Error> for ParseError {
	fn from(_: str::Utf8Error) -> Self { Self::Invalid }
}
