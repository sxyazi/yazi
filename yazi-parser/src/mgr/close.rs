use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

use crate::mgr::QuitOpt;

#[derive(Debug, Default)]
pub struct CloseOpt(pub QuitOpt);

impl From<CmdCow> for CloseOpt {
	fn from(c: CmdCow) -> Self { Self(c.into()) }
}

impl FromLua for CloseOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CloseOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
