use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_core::app::QuitOpt;
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::mlua::SER_OPT;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct QuitForm {
	#[serde(flatten)]
	pub opt: QuitOpt,
}

impl From<QuitOpt> for QuitForm {
	fn from(opt: QuitOpt) -> Self { Self { opt } }
}

impl<C> FromAction<C> for QuitForm {
	fn from_action(mut a: ActionCow, cx: &C) -> anyhow::Result<Self> {
		Ok(Self {
			opt: if let Some(opt) = a.take_any("opt") { opt } else { FromAction::from_action(a, cx)? },
		})
	}
}

impl FromLua for QuitForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for QuitForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
