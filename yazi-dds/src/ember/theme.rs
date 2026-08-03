use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberTheme;

impl EmberTheme {
	pub fn borrowed() -> Ember<'static> { Self.into() }

	pub fn owned() -> Ember<'static> { Self::borrowed() }
}

impl From<EmberTheme> for Ember<'_> {
	fn from(value: EmberTheme) -> Self { Self::Theme(value) }
}

impl IntoLua for EmberTheme {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Ok(Value::Nil) }
}
