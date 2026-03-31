use crossterm::event::MouseEvent;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};

#[derive(Debug)]
pub struct MouseForm {
	pub event: MouseEvent,
}

impl From<MouseEvent> for MouseForm {
	fn from(event: MouseEvent) -> Self { Self { event } }
}

impl FromLua for MouseForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for MouseForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
