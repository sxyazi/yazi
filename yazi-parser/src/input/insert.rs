use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct InsertOpt {
	pub append: bool,
}

impl From<CmdCow> for InsertOpt {
	fn from(c: CmdCow) -> Self { Self { append: c.bool("append") } }
}

impl From<bool> for InsertOpt {
	fn from(append: bool) -> Self { Self { append } }
}

impl FromLua for InsertOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for InsertOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
