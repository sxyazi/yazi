use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct SeekOpt {
	pub units: i16,
}

impl From<ActionCow> for SeekOpt {
	fn from(a: ActionCow) -> Self { Self { units: a.first().unwrap_or(0) } }
}

impl FromLua for SeekOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SeekOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
