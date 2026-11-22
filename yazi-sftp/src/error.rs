use std::{borrow::Cow, io::ErrorKind};

use crate::responses::{self, StatusCode};

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

	pub fn maybe_exists(&self) -> bool {
		match self {
			Self::Status(s) => s.code.is_failure() || s.code.is_file_already_exists(),
			Self::IO(e) => e.kind() == ErrorKind::AlreadyExists,
			_ => false,
		}
	}
}

impl serde::ser::Error for Error {
	fn custom<T: std::fmt::Display>(msg: T) -> Self { Self::serde(msg.to_string()) }
}

impl serde::de::Error for Error {
	fn custom<T: std::fmt::Display>(msg: T) -> Self { Self::serde(msg.to_string()) }
}

impl From<std::str::Utf8Error> for Error {
	fn from(err: std::str::Utf8Error) -> Self { Self::serde(err.to_string()) }
}

impl From<Error> for std::io::Error {
	fn from(err: Error) -> Self {
		match err {
			Error::IO(e) => e,
			Error::Serde(e) => Self::new(ErrorKind::InvalidData, e),
			Error::Status(status) => match status.code {
				StatusCode::Ok => Self::other("unexpected OK"),
				StatusCode::Eof => Self::from(ErrorKind::UnexpectedEof),
				StatusCode::NoSuchFile => Self::from(ErrorKind::NotFound),
				StatusCode::PermissionDenied => Self::from(ErrorKind::PermissionDenied),
				StatusCode::Failure => Self::from(ErrorKind::Other),
				StatusCode::BadMessage => Self::from(ErrorKind::InvalidData),
				StatusCode::NoConnection => Self::from(ErrorKind::NotConnected),
				StatusCode::ConnectionLost => Self::from(ErrorKind::ConnectionReset),
				StatusCode::OpUnsupported => Self::from(ErrorKind::Unsupported),
				StatusCode::InvalidHandle => Self::from(ErrorKind::InvalidInput),
				StatusCode::NoSuchPath => Self::from(ErrorKind::NotFound),
				StatusCode::FileAlreadyExists => Self::from(ErrorKind::AlreadyExists),
				StatusCode::WriteProtect => Self::from(ErrorKind::PermissionDenied),
				StatusCode::NoMedia => Self::from(ErrorKind::Other),
			},
			Error::Packet(e) => Self::new(ErrorKind::InvalidData, e),
			Error::Timeout => Self::from(ErrorKind::TimedOut),
			Error::Unsupported => Self::from(ErrorKind::Unsupported),
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
