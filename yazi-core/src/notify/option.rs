use std::time::Duration;

use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use serde_with::{DurationSecondsWithFrac, serde_as};
use yazi_binding::SER_OPT;
use yazi_shared::event::ActionCow;

use crate::notify::MessageLevel;

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MessageOpt {
	#[serde(alias = "0")]
	pub content: String,
	#[serde(alias = "1")]
	pub title:   String,
	#[serde(default)]
	pub level:   MessageLevel,
	#[serde_as(as = "DurationSecondsWithFrac<f64>")]
	pub timeout: Duration,
}

impl TryFrom<ActionCow> for MessageOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for MessageOpt {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for MessageOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
