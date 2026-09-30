use thiserror::Error;

#[derive(Debug, Error)]
pub enum DateError {
	#[error("timestamp must be finite")]
	NonFinite,
	#[error("timestamp is out of range")]
	OutOfRange,
	#[error("invalid date format")]
	InvalidFormat,
}

impl From<DateError> for mlua::Error {
	fn from(err: DateError) -> Self { Self::external(err) }
}
