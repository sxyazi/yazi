use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct DeleteOpt {
	pub cut:    bool,
	pub insert: bool,
}

impl From<CmdCow> for DeleteOpt {
	fn from(c: CmdCow) -> Self { Self { cut: c.bool("cut"), insert: c.bool("insert") } }
}

impl FromLua for DeleteOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DeleteOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
