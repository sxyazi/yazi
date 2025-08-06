use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct CreateOpt {
	pub dir:   bool,
	pub force: bool,
}

impl From<CmdCow> for CreateOpt {
	fn from(c: CmdCow) -> Self { Self { dir: c.bool("dir"), force: c.bool("force") } }
}

impl FromLua for CreateOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CreateOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
