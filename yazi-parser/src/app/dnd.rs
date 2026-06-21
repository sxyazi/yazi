use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_term::event::DndEvent;

#[derive(Debug)]
pub struct DndForm {
	pub event: DndEvent,
}

impl From<DndEvent> for DndForm {
	fn from(event: DndEvent) -> Self { Self { event } }
}

impl FromLua for DndForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DndForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
