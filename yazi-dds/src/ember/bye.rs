use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberBye;

impl EmberBye {
	pub fn borrowed() -> Ember<'static> { Self.into() }
}

impl From<EmberBye> for Ember<'_> {
	fn from(value: EmberBye) -> Self { Self::Bye(value) }
}

impl IntoLua for EmberBye {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.create_table()?.into_lua(lua) }
}
