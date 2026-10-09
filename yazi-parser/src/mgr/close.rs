use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_core::app::QuitOpt;
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::mlua::SER_OPT;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CloseForm {
	#[serde(flatten)]
	pub opt: QuitOpt,
}

impl<C> FromAction<C> for CloseForm {
	fn from_action(mut a: ActionCow, cx: &C) -> anyhow::Result<Self> {
		Ok(Self {
			opt: if let Some(opt) = a.take_any("opt") { opt } else { FromAction::from_action(a, cx)? },
		})
	}
}

impl FromLua for CloseForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for CloseForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
