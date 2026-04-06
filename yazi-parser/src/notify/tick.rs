use std::time::Duration;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use serde_with::{DurationSecondsWithFrac, serde_as};
use yazi_shared::event::ActionCow;

#[serde_as]
#[derive(Debug, Default, Deserialize)]
pub struct TickForm {
	#[serde(alias = "0")]
	#[serde_as(as = "DurationSecondsWithFrac<f64>")]
	pub interval: Duration,
}

impl TryFrom<ActionCow> for TickForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for TickForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TickForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
