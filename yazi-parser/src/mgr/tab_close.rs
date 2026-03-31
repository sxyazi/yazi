use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct TabCloseForm {
	pub idx: usize,
}

impl From<ActionCow> for TabCloseForm {
	fn from(a: ActionCow) -> Self { Self { idx: a.first().unwrap_or(0) } }
}

impl From<usize> for TabCloseForm {
	fn from(idx: usize) -> Self { Self { idx } }
}

impl FromLua for TabCloseForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabCloseForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
