use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct KillOpt {
	pub kind: SStr,
}

impl From<CmdCow> for KillOpt {
	fn from(mut c: CmdCow) -> Self { Self { kind: c.take_first().unwrap_or_default() } }
}

impl FromLua for KillOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for KillOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
