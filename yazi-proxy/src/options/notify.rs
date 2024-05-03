use std::{str::FromStr, time::Duration};

use anyhow::bail;
use mlua::{ExternalError, ExternalResult};
use yazi_config::THEME;
use yazi_shared::{event::Cmd, theme::Style};

pub struct NotifyOpt {
	pub title:   String,
	pub content: String,
	pub level:   NotifyLevel,
	pub timeout: Duration,
}

impl TryFrom<Cmd> for NotifyOpt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> { c.take_any("option").ok_or(()) }
}

impl<'a> TryFrom<mlua::Table<'a>> for NotifyOpt {
	type Error = mlua::Error;

	fn try_from(t: mlua::Table) -> Result<Self, Self::Error> {
		let timeout = t.raw_get::<_, f64>("timeout")?;
		if timeout < 0.0 {
			return Err("timeout must be non-negative".into_lua_err());
		}

		let level = if let Ok(s) = t.raw_get::<_, mlua::String>("level") {
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

#[derive(Clone, Copy, Default)]
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
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"info" => Self::Info,
			"warn" => Self::Warn,
			"error" => Self::Error,
			_ => bail!("Invalid notify level: {s}"),
		})
	}
}
