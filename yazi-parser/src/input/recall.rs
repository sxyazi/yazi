use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::{ActionCow, FromAction};
use yazi_widgets::Step;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct RecallForm {
	#[serde(alias = "0")]
	pub step: Step,
}

impl<C> FromAction<C> for RecallForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for RecallForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RecallForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
