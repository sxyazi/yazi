use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::mlua::SER_OPT;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct HiddenForm {
	#[serde(alias = "0")]
	pub state: HiddenFormState,
}

impl<C> FromAction<C> for HiddenForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for HiddenForm {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for HiddenForm {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}

// --- State
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HiddenFormState {
	#[default]
	None,
	Show,
	Hide,
	Toggle,
}

impl HiddenFormState {
	pub fn bool(self, old: bool) -> bool {
		match self {
			Self::None => old,
			Self::Show => true,
			Self::Hide => false,
			Self::Toggle => !old,
		}
	}
}
