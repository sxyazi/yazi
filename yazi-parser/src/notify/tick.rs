use std::time::Duration;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use serde_with::{DurationSecondsWithFrac, serde_as};
use yazi_shared::event::{ActionCow, FromAction};

#[serde_as]
#[derive(Debug, Default, Deserialize)]
pub struct TickForm {
	#[serde(alias = "0")]
	#[serde_as(as = "DurationSecondsWithFrac<f64>")]
	pub interval: Duration,
}

impl<C> FromAction<C> for TickForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for TickForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TickForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
