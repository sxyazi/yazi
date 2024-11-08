use mlua::{IntoLua, Lua, Value};
use serde::Serialize;
use yazi_shared::event::Data;

use super::Body;
use crate::Sendable;

#[derive(Debug)]
pub struct BodyCustom {
	pub kind: String,
	pub data: Data,
}

impl BodyCustom {
	#[inline]
	pub fn from_str(kind: &str, data: &str) -> anyhow::Result<Body<'static>> {
		Ok(Self { kind: kind.to_owned(), data: serde_json::from_str(data)? }.into())
	}

	#[inline]
	pub fn from_lua(kind: &str, data: Value) -> mlua::Result<Body<'static>> {
		Ok(Self { kind: kind.to_owned(), data: Sendable::value_to_data(data)? }.into())
	}
}

impl From<BodyCustom> for Body<'_> {
	fn from(value: BodyCustom) -> Self { Self::Custom(value) }
}

impl IntoLua for BodyCustom {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { Sendable::data_to_value(lua, self.data) }
}

impl Serialize for BodyCustom {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serde::Serialize::serialize(&self.data, serializer)
	}
}
