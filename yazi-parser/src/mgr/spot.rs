use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug, Default)]
pub struct SpotOpt {
	pub skip: Option<usize>,
}

impl From<CmdCow> for SpotOpt {
	fn from(c: CmdCow) -> Self { Self { skip: c.get("skip").ok() } }
}

impl From<usize> for SpotOpt {
	fn from(skip: usize) -> Self { Self { skip: Some(skip) } }
}

impl FromLua for SpotOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SpotOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
