use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct SelectForm {
	pub index: usize,
}

impl From<ActionCow> for SelectForm {
	fn from(c: ActionCow) -> Self { Self { index: c.first().unwrap_or(0) } }
}

impl From<usize> for SelectForm {
	fn from(index: usize) -> Self { Self { index } }
}

impl FromLua for SelectForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SelectForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
