use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Ember;

#[derive(Debug, Deserialize, Serialize)]
pub struct EmberMount;

impl EmberMount {
	pub fn owned() -> Ember<'static> { Self.into() }

	pub fn borrowed() -> Ember<'static> { Self::owned() }
}

impl From<EmberMount> for Ember<'_> {
	fn from(value: EmberMount) -> Self { Self::Mount(value) }
}

impl IntoLua for EmberMount {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Ok(Value::Nil) }
}
