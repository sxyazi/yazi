use mlua::{IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_shim::mlua::SER_OPT;

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberFocus {
	lost: bool,
}

impl EmberFocus {
	pub(crate) fn borrowed(lost: bool) -> Ember<'static> { Self { lost }.into() }

	pub(crate) fn owned(lost: bool) -> Ember<'static> { Self::borrowed(lost) }
}

impl From<EmberFocus> for Ember<'_> {
	fn from(value: EmberFocus) -> Self { Self::Focus(value) }
}

impl IntoLua for EmberFocus {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
