use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::SStr;

#[derive(Debug, Deserialize)]
pub struct RenameForm {
	#[serde(default)]
	pub hovered: bool,
	#[serde(default)]
	pub force:   bool,
	#[serde(default)]
	pub empty:   SStr,
	#[serde(default)]
	pub cursor:  SStr,
}

impl<C> FromAction<C> for RenameForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for RenameForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for RenameForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
