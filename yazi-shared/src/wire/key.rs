use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, de::{self, IntoDeserializer}};
use strum::EnumIs;

use super::Wire;

#[derive(Clone, Debug, Deserialize, EnumIs, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum WireKey {
	Integer(i64),
	String(String),
}

impl From<i64> for WireKey {
	fn from(value: i64) -> Self { Self::Integer(value) }
}

impl From<String> for WireKey {
	fn from(value: String) -> Self { Self::String(value) }
}

impl From<WireKey> for Wire {
	fn from(value: WireKey) -> Self {
		match value {
			WireKey::Integer(i) => Self::Integer(i),
			WireKey::String(s) => Self::String(s),
		}
	}
}

impl<'de> IntoDeserializer<'de, de::value::Error> for WireKey {
	type Deserializer = Wire;

	fn into_deserializer(self) -> Self::Deserializer { self.into() }
}

impl FromLua for WireKey {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Integer(i) => Ok(i.into()),
			Value::String(s) => Ok(s.to_str()?.to_owned().into()),
			_ => Err("unsupported value in wire data".into_lua_err()),
		}
	}
}

impl IntoLua for WireKey {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Ok(match self {
			Self::Integer(i) => Value::Integer(i),
			Self::String(s) => Value::String(lua.create_external_string(s)?),
		})
	}
}

impl IntoLua for &WireKey {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Ok(match self {
			WireKey::Integer(i) => Value::Integer(*i),
			WireKey::String(s) => Value::String(lua.create_string(s)?),
		})
	}
}
