use std::fmt::Debug;

use anyhow::Result;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::ActionCow};

#[derive(Clone, Debug, Default)]
pub struct LuaForm {
	pub code: SStr,
}

impl TryFrom<ActionCow> for LuaForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> { Ok(Self { code: a.take_first()? }) }
}

impl FromLua for LuaForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for LuaForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
