use mlua::{IntoLua, Lua, Value};
use serde::Serialize;
use yazi_shared::data::Data;

use super::Ember;
use crate::Sendable;

#[derive(Debug)]
pub struct EmberCustom {
	pub kind: String,
	pub data: Data,
}

impl EmberCustom {
	pub fn from_str(kind: &str, data: &str) -> anyhow::Result<Ember<'static>> {
		Ok(Self { kind: kind.to_owned(), data: serde_json::from_str(data)? }.into())
	}

	pub fn from_lua(lua: &Lua, kind: &str, data: Value) -> mlua::Result<Ember<'static>> {
		Ok(Self { kind: kind.to_owned(), data: Sendable::value_to_data(lua, data)? }.into())
	}
}

impl From<EmberCustom> for Ember<'_> {
	fn from(value: EmberCustom) -> Self { Self::Custom(value) }
}

impl IntoLua for EmberCustom {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { Sendable::data_to_value(lua, self.data) }
}

impl Serialize for EmberCustom {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serde::Serialize::serialize(&self.data, serializer)
	}
}
