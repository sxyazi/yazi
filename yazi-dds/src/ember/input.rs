use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberInput<'a> {
	r#type: Cow<'a, str>,
	value:  Cow<'a, str>,
}

impl<'a> EmberInput<'a> {
	pub(crate) fn borrowed(r#type: &'a str, value: &'a str) -> Ember<'a> {
		Self { r#type: r#type.into(), value: value.into() }.into()
	}
}

impl EmberInput<'static> {
	pub(crate) fn owned(r#type: &'static str, value: &str) -> Ember<'static> {
		Self { r#type: r#type.into(), value: value.to_owned().into() }.into()
	}
}

impl<'a> From<EmberInput<'a>> for Ember<'a> {
	fn from(value: EmberInput<'a>) -> Self { Self::Input(value) }
}

impl IntoLua for EmberInput<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("type", self.r#type), ("value", self.value)])?.into_lua(lua)
	}
}
