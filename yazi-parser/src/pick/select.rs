use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct SelectOpt {
	pub index: usize,
}

impl From<CmdCow> for SelectOpt {
	fn from(c: CmdCow) -> Self { Self { index: c.first().unwrap_or(0) } }
}

impl From<usize> for SelectOpt {
	fn from(index: usize) -> Self { Self { index } }
}

impl FromLua for SelectOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SelectOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
