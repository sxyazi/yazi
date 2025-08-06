use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug, Default)]
pub struct BackspaceOpt {
	pub under: bool,
}

impl From<CmdCow> for BackspaceOpt {
	fn from(c: CmdCow) -> Self { Self { under: c.bool("under") } }
}

impl From<bool> for BackspaceOpt {
	fn from(under: bool) -> Self { Self { under } }
}

impl FromLua for BackspaceOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for BackspaceOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
