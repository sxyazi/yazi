use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct VisualModeForm {
	pub unset: bool,
}

impl From<ActionCow> for VisualModeForm {
	fn from(a: ActionCow) -> Self { Self { unset: a.bool("unset") } }
}

impl FromLua for VisualModeForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for VisualModeForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
