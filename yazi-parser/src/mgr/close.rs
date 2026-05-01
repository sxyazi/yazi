use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::SER_OPT;
use yazi_core::app::QuitOpt;
use yazi_shared::event::ActionCow;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CloseForm {
	#[serde(flatten)]
	pub opt: QuitOpt,
}

impl TryFrom<ActionCow> for CloseForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: if let Some(opt) = a.take_any("opt") { opt } else { a.try_into()? } })
	}
}

impl FromLua for CloseForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for CloseForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
