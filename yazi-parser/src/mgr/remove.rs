use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug, Deserialize)]
pub struct RemoveForm {
	#[serde(default)]
	pub force:       bool,
	#[serde(default)]
	pub permanently: bool,
	#[serde(default)]
	pub hovered:     bool,
}

impl<C> FromAction<C> for RemoveForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for RemoveForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RemoveForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
