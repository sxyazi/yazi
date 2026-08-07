use std::fmt::{Display, Formatter};

use crate::cell::SyncCell;

pub static LOG_LEVEL: SyncCell<LogLevel> = SyncCell::new(LogLevel::None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
	None,
	Error,
	Warn,
	Info,
	Debug,
}

impl LogLevel {
	#[inline]
	pub fn is_none(self) -> bool { self == Self::None }
}

impl From<String> for LogLevel {
	fn from(mut s: String) -> Self {
		s.make_ascii_uppercase();
		match s.as_str() {
			"ERROR" => Self::Error,
			"WARN" => Self::Warn,
			"INFO" => Self::Info,
			"DEBUG" => Self::Debug,
			_ => Self::None,
		}
	}
}

impl AsRef<str> for LogLevel {
	fn as_ref(&self) -> &str {
		match self {
			Self::None => "yazi=NONE",
			Self::Error => "yazi=ERROR",
			Self::Warn => "yazi=WARN",
			Self::Info => "yazi=INFO",
			Self::Debug => "yazi=DEBUG",
		}
	}
}

impl Display for LogLevel {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.as_ref()) }
}
