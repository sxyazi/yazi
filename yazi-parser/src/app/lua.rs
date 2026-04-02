use std::fmt::Debug;

use anyhow::Result;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{SStr, event::ActionCow};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LuaForm {
	#[serde(alias = "0")]
	pub code: SStr,
}

impl TryFrom<ActionCow> for LuaForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for LuaForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for LuaForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
