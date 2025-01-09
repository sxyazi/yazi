use std::fmt::{Display, Formatter};

pub static LOG_LEVEL: crate::SyncCell<LogLevel> = crate::SyncCell::new(LogLevel::None);

#[inline]
pub fn env_exists(name: &str) -> bool { std::env::var_os(name).is_some_and(|s| !s.is_empty()) }

#[inline]
pub fn in_wsl() -> bool {
	#[cfg(target_os = "linux")]
	{
		std::fs::symlink_metadata("/proc/sys/fs/binfmt_misc/WSLInterop").is_ok()
	}
	#[cfg(not(target_os = "linux"))]
	{
		false
	}
}

#[inline]
pub fn in_ssh_connection() -> bool {
	env_exists("SSH_CLIENT") || env_exists("SSH_TTY") || env_exists("SSH_CONNECTION")
}

// LogLevel
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
			Self::None => "NONE",
			Self::Error => "ERROR",
			Self::Warn => "WARN",
			Self::Info => "INFO",
			Self::Debug => "DEBUG",
		}
	}
}

impl Display for LogLevel {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.as_ref()) }
}
