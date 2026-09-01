use std::borrow::Cow;

use mlua::{IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberHistory<'a> {
	pub group: Cow<'a, str>,
	pub value: Cow<'a, str>,
}

impl<'a> EmberHistory<'a> {
	pub fn borrowed(group: &'a str, value: &'a str) -> Ember<'a> {
		Self { group: group.into(), value: value.into() }.into()
	}
}

impl EmberHistory<'static> {
	pub fn owned(group: &str, value: &str) -> Ember<'static> {
		Self { group: group.to_owned().into(), value: value.to_owned().into() }.into()
	}
}

impl<'a> From<EmberHistory<'a>> for Ember<'a> {
	fn from(value: EmberHistory<'a>) -> Self { Self::History(value) }
}

impl IntoLua for EmberHistory<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("group", self.group), ("value", self.value)])?.into_lua(lua)
	}
}
