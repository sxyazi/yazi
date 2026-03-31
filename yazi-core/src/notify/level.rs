use serde::{Deserialize, Serialize};
use yazi_config::{Style, THEME};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageLevel {
	#[default]
	Info,
	Warn,
	Error,
}

impl MessageLevel {
	pub fn icon(self) -> &'static str {
		match self {
			Self::Info => &THEME.notify.icon_info,
			Self::Warn => &THEME.notify.icon_warn,
			Self::Error => &THEME.notify.icon_error,
		}
	}

	pub fn style(self) -> Style {
		match self {
			Self::Info => THEME.notify.title_info,
			Self::Warn => THEME.notify.title_warn,
			Self::Error => THEME.notify.title_error,
		}
	}
}
