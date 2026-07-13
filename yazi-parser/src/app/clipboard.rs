use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_term::event::ClipboardEvent;

#[derive(Debug)]
pub struct ClipboardForm {
	pub event: ClipboardEvent,
}

impl From<ClipboardEvent> for ClipboardForm {
	fn from(event: ClipboardEvent) -> Self { Self { event } }
}

impl FromLua for ClipboardForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ClipboardForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
