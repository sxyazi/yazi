use std::sync::Arc;

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
	pub fn icon(self) -> Arc<String> {
		match self {
			Self::Info => THEME.notify.icon_info.load_full(),
			Self::Warn => THEME.notify.icon_info.load_full(),
			Self::Error => THEME.notify.icon_info.load_full(),
		}
	}

	pub fn style(self) -> Style {
		match self {
			Self::Info => THEME.notify.title_info.get(),
			Self::Warn => THEME.notify.title_warn.get(),
			Self::Error => THEME.notify.title_error.get(),
		}
	}
}
