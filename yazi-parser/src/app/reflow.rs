use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use ratatui::layout::Rect;

#[derive(Debug)]
pub struct ReflowOpt {
	pub reflow: fn(Rect) -> mlua::Result<Table>,
}

impl From<fn(Rect) -> mlua::Result<Table>> for ReflowOpt {
	fn from(f: fn(Rect) -> mlua::Result<Table>) -> Self { Self { reflow: f } }
}

impl FromLua for ReflowOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ReflowOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
