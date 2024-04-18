use std::{str::FromStr, time::Duration};

use anyhow::bail;
use mlua::{ExternalError, ExternalResult};
use yazi_shared::event::Cmd;

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

#[derive(Default)]
pub enum NotifyLevel {
	#[default]
	Info,
	Warn,
	Error,
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
