use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberBye;

impl EmberBye {
	pub fn owned() -> Ember<'static> { Self.into() }
}

impl From<EmberBye> for Ember<'_> {
	fn from(value: EmberBye) -> Self { Self::Bye(value) }
}

impl IntoLua for EmberBye {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
		Err("BodyBye cannot be converted to Lua").into_lua_err()
	}
}
