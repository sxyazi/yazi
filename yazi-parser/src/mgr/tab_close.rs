use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::event::{CmdCow, Data};

#[derive(Debug)]
pub struct TabCloseOpt {
	pub idx: usize,
}

impl From<CmdCow> for TabCloseOpt {
	fn from(c: CmdCow) -> Self { Self { idx: c.first().and_then(Data::as_usize).unwrap_or(0) } }
}

impl From<usize> for TabCloseOpt {
	fn from(idx: usize) -> Self { Self { idx } }
}

impl IntoLua for &TabCloseOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
