use std::{fmt, ops::Deref};

use mlua::{FromLua, IntoLua, Lua, Value};

#[derive(Clone)]
pub struct ByteString(mlua::String);

impl Deref for ByteString {
	type Target = mlua::String;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl fmt::Display for ByteString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.display().fmt(f) }
}

impl FromLua for ByteString {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self(mlua::String::from_lua(value, lua)?))
	}
}

impl IntoLua for ByteString {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.0.into_lua(lua) }
}
