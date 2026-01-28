use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

use crate::app::QuitOpt;

#[derive(Debug, Default)]
pub struct CloseOpt(pub QuitOpt);

impl TryFrom<CmdCow> for CloseOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> { c.try_into().map(Self) }
}

impl FromLua for CloseOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CloseOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
