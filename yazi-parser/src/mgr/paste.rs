use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct PasteOpt {
	pub force:  bool,
	pub follow: bool,
}

impl From<CmdCow> for PasteOpt {
	fn from(c: CmdCow) -> Self { Self { force: c.bool("force"), follow: c.bool("follow") } }
}

impl FromLua for PasteOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PasteOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
