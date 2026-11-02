use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::mlua::SER_OPT;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FocusForm {
	#[serde(default)]
	pub lost: bool,
}

impl<C> FromAction<C> for FocusForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl From<bool> for FocusForm {
	fn from(lost: bool) -> Self { Self { lost } }
}

impl FromLua for FocusForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for FocusForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
