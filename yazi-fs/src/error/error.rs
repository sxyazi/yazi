use std::{fmt, io, sync::Arc};

use anyhow::Result;

use crate::error::{kind_from_str, kind_to_str};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
	Kind(io::ErrorKind),
	Raw(i32),
	Custom { kind: io::ErrorKind, code: Option<i32>, message: Arc<str> },
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self {
		if err.get_ref().is_some() {
			Self::Custom {
				kind:    err.kind(),
				code:    err.raw_os_error(),
				message: err.to_string().into(),
			}
		} else if let Some(code) = err.raw_os_error() {
			Self::Raw(code)
		} else {
			Self::Kind(err.kind())
		}
	}
}

impl From<io::ErrorKind> for Error {
	fn from(kind: io::ErrorKind) -> Self { Self::Kind(kind) }
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Kind(kind) => io::Error::from(*kind).fmt(f),
			Self::Raw(code) => io::Error::from_raw_os_error(*code).fmt(f),
			Self::Custom { message, .. } => write!(f, "{message}"),
		}
	}
}

impl Error {
	pub fn custom(kind: &str, code: Option<i32>, message: &str) -> Result<Self> {
		Ok(Self::Custom { kind: kind_from_str(kind)?, code, message: message.into() })
	}

	pub fn kind(&self) -> io::ErrorKind {
		match self {
			Self::Kind(kind) => *kind,
			Self::Raw(code) => io::Error::from_raw_os_error(*code).kind(),
			Self::Custom { kind, .. } => *kind,
		}
	}

	pub fn kind_str(&self) -> &'static str { kind_to_str(self.kind()) }

	pub fn raw_os_error(&self) -> Option<i32> {
		match self {
			Self::Kind(_) => None,
			Self::Raw(code) => Some(*code),
			Self::Custom { code, .. } => *code,
		}
	}
}
