use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct CopyOpt {
	pub r#type: SStr,
}

impl From<CmdCow> for CopyOpt {
	fn from(mut c: CmdCow) -> Self { Self { r#type: c.take_first().unwrap_or_default() } }
}

impl FromLua for CopyOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CopyOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
