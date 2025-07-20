use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct YankOpt {
	pub cut: bool,
}

impl From<CmdCow> for YankOpt {
	fn from(c: CmdCow) -> Self { Self { cut: c.bool("cut") } }
}

impl IntoLua for &YankOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
