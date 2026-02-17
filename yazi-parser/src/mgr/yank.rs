use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct YankOpt {
	pub cut: bool,
}

impl From<ActionCow> for YankOpt {
	fn from(a: ActionCow) -> Self { Self { cut: a.bool("cut") } }
}

impl FromLua for YankOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for YankOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
