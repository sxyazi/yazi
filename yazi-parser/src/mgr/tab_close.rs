use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct TabCloseOpt {
	pub idx: usize,
}

impl From<CmdCow> for TabCloseOpt {
	fn from(c: CmdCow) -> Self { Self { idx: c.first().unwrap_or(0) } }
}

impl From<usize> for TabCloseOpt {
	fn from(idx: usize) -> Self { Self { idx } }
}

impl FromLua for TabCloseOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabCloseOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
