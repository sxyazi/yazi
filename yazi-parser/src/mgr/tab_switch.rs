use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct TabSwitchOpt {
	pub step:     isize,
	pub relative: bool,
}

impl From<ActionCow> for TabSwitchOpt {
	fn from(a: ActionCow) -> Self {
		Self { step: a.first().unwrap_or(0), relative: a.bool("relative") }
	}
}

impl FromLua for TabSwitchOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabSwitchOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
