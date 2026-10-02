use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimeError {
	#[error("timestamp must be finite")]
	NonFinite,
	#[error("timestamp is out of range")]
	OutOfRange,
	#[error("invalid time format")]
	InvalidFormat,
}

impl From<std::time::TryFromFloatSecsError> for TimeError {
	fn from(_: std::time::TryFromFloatSecsError) -> Self { Self::OutOfRange }
}

impl From<chrono::OutOfRangeError> for TimeError {
	fn from(_: chrono::OutOfRangeError) -> Self { Self::OutOfRange }
}

impl From<std::fmt::Error> for TimeError {
	fn from(_: std::fmt::Error) -> Self { Self::InvalidFormat }
}

impl From<yazi_ffi::locale::LocaleError> for TimeError {
	fn from(_: yazi_ffi::locale::LocaleError) -> Self { Self::InvalidFormat }
}

impl From<TimeError> for mlua::Error {
	fn from(err: TimeError) -> Self { Self::external(err) }
}
