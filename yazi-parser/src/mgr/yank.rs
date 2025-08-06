use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct YankOpt {
	pub cut: bool,
}

impl From<CmdCow> for YankOpt {
	fn from(c: CmdCow) -> Self { Self { cut: c.bool("cut") } }
}

impl FromLua for YankOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for YankOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
