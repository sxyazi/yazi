use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug, Default)]
pub struct CloseForm {
	pub submit: bool,
}

impl From<ActionCow> for CloseForm {
	fn from(a: ActionCow) -> Self { Self { submit: a.bool("submit") } }
}

impl From<bool> for CloseForm {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl FromLua for CloseForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CloseForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
