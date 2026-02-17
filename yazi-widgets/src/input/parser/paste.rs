use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct PasteOpt {
	pub before: bool,
}

impl From<ActionCow> for PasteOpt {
	fn from(a: ActionCow) -> Self { Self { before: a.bool("before") } }
}

impl FromLua for PasteOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PasteOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
