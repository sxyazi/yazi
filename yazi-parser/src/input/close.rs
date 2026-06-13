use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::SER_OPT;
use yazi_shared::event::ActionCow;

#[derive(Debug, Default, Deserialize, Serialize)]
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
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for CloseForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
