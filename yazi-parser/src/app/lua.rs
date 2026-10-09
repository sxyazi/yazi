use std::fmt::Debug;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::SStr;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LuaForm {
	#[serde(alias = "0")]
	pub code: SStr,
}

impl<C> FromAction<C> for LuaForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for LuaForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for LuaForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
