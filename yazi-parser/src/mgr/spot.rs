use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::event::{CmdCow, Data};

#[derive(Debug, Default)]
pub struct SpotOpt {
	pub skip: Option<usize>,
}

impl From<CmdCow> for SpotOpt {
	fn from(c: CmdCow) -> Self { Self { skip: c.get("skip").and_then(Data::as_usize) } }
}

impl From<usize> for SpotOpt {
	fn from(skip: usize) -> Self { Self { skip: Some(skip) } }
}

impl IntoLua for &SpotOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
