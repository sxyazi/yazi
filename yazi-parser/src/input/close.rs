use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug, Default)]
pub struct CloseOpt {
	pub submit: bool,
}

impl From<ActionCow> for CloseOpt {
	fn from(a: ActionCow) -> Self { Self { submit: a.bool("submit") } }
}

impl From<bool> for CloseOpt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl FromLua for CloseOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CloseOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
