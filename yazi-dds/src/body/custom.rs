use mlua::{IntoLua, Lua, Value};
use serde::Serialize;

use super::Body;
use crate::ValueSendable;

#[derive(Debug)]
pub struct BodyCustom {
	pub kind:  String,
	pub value: ValueSendable,
}

impl BodyCustom {
	#[inline]
	pub fn from_str(kind: &str, value: &str) -> anyhow::Result<Body<'static>> {
		Ok(Self { kind: kind.to_owned(), value: serde_json::from_str(value)? }.into())
	}

	#[inline]
	pub fn from_lua(kind: &str, value: Value) -> mlua::Result<Body<'static>> {
		Ok(Self { kind: kind.to_owned(), value: value.try_into()? }.into())
	}
}

impl From<BodyCustom> for Body<'_> {
	fn from(value: BodyCustom) -> Self { Self::Custom(value) }
}

impl IntoLua<'_> for BodyCustom {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.value.into_lua(lua) }
}

impl Serialize for BodyCustom {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serde::Serialize::serialize(&self.value, serializer)
	}
}
