use std::{str::FromStr, time::Duration};

use mlua::{ExternalError, ExternalResult};
use serde::Deserialize;
use yazi_config::THEME;
use yazi_shared::{event::CmdCow, theme::Style};

pub struct NotifyOpt {
	pub title:   String,
	pub content: String,
	pub level:   NotifyLevel,
	pub timeout: Duration,
}

impl TryFrom<CmdCow> for NotifyOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> { c.take_any("option").ok_or(()) }
}

impl TryFrom<mlua::Table> for NotifyOpt {
	type Error = mlua::Error;

	fn try_from(t: mlua::Table) -> Result<Self, Self::Error> {
		let timeout = t.raw_get("timeout")?;
		if timeout < 0.0 {
			return Err("timeout must be non-negative".into_lua_err());
		}

		let level = if let Ok(s) = t.raw_get::<mlua::String>("level") {
			s.to_str()?.parse().into_lua_err()?
		} else {
			Default::default()
		};

		Ok(Self {
			title: t.raw_get("title")?,
			content: t.raw_get("content")?,
			level,
			timeout: Duration::from_secs_f64(timeout),
		})
	}
}

#[derive(Clone, Copy, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum NotifyLevel {
	#[default]
	Info,
	Warn,
	Error,
}

impl NotifyLevel {
	#[inline]
	pub fn icon(self) -> &'static str {
		match self {
			Self::Info => &THEME.notify.icon_info,
			Self::Warn => &THEME.notify.icon_warn,
			Self::Error => &THEME.notify.icon_error,
		}
	}

	#[inline]
	pub fn style(self) -> &'static Style {
		match self {
			Self::Info => &THEME.notify.title_info,
			Self::Warn => &THEME.notify.title_warn,
			Self::Error => &THEME.notify.title_error,
		}
	}
}

impl FromStr for NotifyLevel {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}
