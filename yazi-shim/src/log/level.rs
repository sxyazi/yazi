use strum::{AsRefStr, Display, EnumIs, EnumString};

use crate::cell::SyncCell;

pub static LOG_LEVEL: SyncCell<LogLevel> = SyncCell::new(LogLevel::None);

#[derive(AsRefStr, Debug, Clone, Copy, Default, Display, EnumIs, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "UPPERCASE", ascii_case_insensitive, prefix = "yazi=")]
pub enum LogLevel {
	#[default]
	None,
	Error,
	Warn,
	Info,
	Debug,
}

impl From<String> for LogLevel {
	fn from(s: String) -> Self { s.parse().unwrap_or_default() }
}
