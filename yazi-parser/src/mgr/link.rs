use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct LinkOpt {
	pub relative: bool,
	pub force:    bool,
}

impl From<CmdCow> for LinkOpt {
	fn from(c: CmdCow) -> Self { Self { relative: c.bool("relative"), force: c.bool("force") } }
}

impl IntoLua for &LinkOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
