use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct LinkForm {
	pub relative: bool,
	pub force:    bool,
}

impl From<ActionCow> for LinkForm {
	fn from(a: ActionCow) -> Self { Self { relative: a.bool("relative"), force: a.bool("force") } }
}

impl FromLua for LinkForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for LinkForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
