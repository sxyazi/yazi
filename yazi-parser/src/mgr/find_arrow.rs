use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct FindArrowOpt {
	pub prev: bool,
}

impl From<CmdCow> for FindArrowOpt {
	fn from(c: CmdCow) -> Self { Self { prev: c.bool("previous") } }
}

impl FromLua for FindArrowOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for FindArrowOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
