use std::{str::FromStr, time::Duration};

use anyhow::anyhow;
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use serde_with::{DurationSeconds, serde_as};
use yazi_config::{Style, THEME};
use yazi_shared::event::CmdCow;

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PushOpt {
	pub title:   String,
	pub content: String,
	pub level:   PushLevel,
	#[serde_as(as = "DurationSeconds<f64>")] // FIXME
	pub timeout: Duration,
}

impl TryFrom<CmdCow> for PushOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		c.take_any("opt").ok_or_else(|| anyhow!("Invalid 'opt' in NotifyOpt"))
	}
}

impl FromLua for PushOpt {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for PushOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value(&self) }
}

// --- Level
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PushLevel {
	#[default]
	Info,
	Warn,
	Error,
}

impl PushLevel {
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

impl FromStr for PushLevel {
	type Err = serde::de::value::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::deserialize(serde::de::value::StrDeserializer::new(s))
	}
}
