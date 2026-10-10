use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug, Default)]
pub struct ExcludeAddForm {
	pub patterns: Vec<String>,
}

impl From<ActionCow> for ExcludeAddForm {
	fn from(mut a: ActionCow) -> Self { Self { patterns: a.take_seq() } }
}

impl FromLua for ExcludeAddForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ExcludeAddForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
