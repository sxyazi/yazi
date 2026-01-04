use std::borrow::Cow;

use crate::responses;

#[derive(Debug)]
pub enum Error {
	IO(std::io::Error),
	Serde(Cow<'static, str>),
	Status(responses::Status),
	Packet(&'static str),
	Timeout,
	Unsupported,
	Custom(Cow<'static, str>),
}

impl Error {
	pub(super) fn serde(s: impl Into<Cow<'static, str>>) -> Self { Self::Serde(s.into()) }

	pub(super) fn custom(s: impl Into<Cow<'static, str>>) -> Self { Self::Custom(s.into()) }
}

impl serde::ser::Error for Error {
	fn custom<T: std::fmt::Display>(msg: T) -> Self { Self::serde(msg.to_string()) }
}

impl serde::de::Error for Error {
	fn custom<T: std::fmt::Display>(msg: T) -> Self { Self::serde(msg.to_string()) }
}

impl From<Error> for std::io::Error {
	fn from(err: Error) -> Self {
		match err {
			Error::IO(e) => e,
			Error::Serde(e) => Self::new(std::io::ErrorKind::InvalidData, e),
			Error::Status(status) => match status.code {
				responses::StatusCode::Ok => Self::other("unexpected OK"),
				responses::StatusCode::Eof => Self::from(std::io::ErrorKind::UnexpectedEof),
				responses::StatusCode::NoSuchFile => Self::from(std::io::ErrorKind::NotFound),
				responses::StatusCode::PermissionDenied => Self::from(std::io::ErrorKind::PermissionDenied),
				responses::StatusCode::Failure => Self::from(std::io::ErrorKind::Other),
				responses::StatusCode::BadMessage => Self::from(std::io::ErrorKind::InvalidData),
				responses::StatusCode::NoConnection => Self::from(std::io::ErrorKind::NotConnected),
				responses::StatusCode::ConnectionLost => Self::from(std::io::ErrorKind::ConnectionReset),
				responses::StatusCode::OpUnsupported => Self::from(std::io::ErrorKind::Unsupported),
				responses::StatusCode::InvalidHandle => Self::from(std::io::ErrorKind::InvalidInput),
				responses::StatusCode::NoSuchPath => Self::from(std::io::ErrorKind::NotFound),
				responses::StatusCode::FileAlreadyExists => Self::from(std::io::ErrorKind::AlreadyExists),
				responses::StatusCode::WriteProtect => Self::from(std::io::ErrorKind::PermissionDenied),
				responses::StatusCode::NoMedia => Self::from(std::io::ErrorKind::Other),
			},
			Error::Packet(e) => Self::new(std::io::ErrorKind::InvalidData, e),
			Error::Timeout => Self::from(std::io::ErrorKind::TimedOut),
			Error::Unsupported => Self::from(std::io::ErrorKind::Unsupported),
			Error::Custom(_) => Self::other(err),
		}
	}
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
	fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self { Self::custom("channel closed") }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
	fn from(_: tokio::sync::oneshot::error::RecvError) -> Self { Self::custom("channel closed") }
}

impl From<tokio::time::error::Elapsed> for Error {
	fn from(_: tokio::time::error::Elapsed) -> Self { Self::Timeout }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IO(e) => write!(f, "IO error: {e}"),
			Self::Serde(s) => write!(f, "Serde error: {s}"),
			Self::Status(s) => write!(f, "Status error: {s:?}"),
			Self::Packet(s) => write!(f, "Unexpected packet: {s}"),
			Self::Timeout => write!(f, "Operation timed out"),
			Self::Unsupported => write!(f, "Operation not supported"),
			Self::Custom(s) => write!(f, "{s}"),
		}
	}
}
