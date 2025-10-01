use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct TabSwitchOpt {
	pub step:     isize,
	pub relative: bool,
}

impl From<CmdCow> for TabSwitchOpt {
	fn from(c: CmdCow) -> Self { Self { step: c.first().unwrap_or(0), relative: c.bool("relative") } }
}

impl FromLua for TabSwitchOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabSwitchOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
