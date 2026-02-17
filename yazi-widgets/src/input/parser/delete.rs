use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct DeleteOpt {
	pub cut:    bool,
	pub insert: bool,
}

impl From<ActionCow> for DeleteOpt {
	fn from(a: ActionCow) -> Self { Self { cut: a.bool("cut"), insert: a.bool("insert") } }
}

impl FromLua for DeleteOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DeleteOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
