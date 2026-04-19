use std::ops::Deref;

use mlua::{FromLua, Lua, LuaSerdeExt, UserData, Value};

pub struct Selector(yazi_config::Selector);

impl Deref for Selector {
	type Target = yazi_config::Selector;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Selector> for yazi_config::Selector {
	fn from(value: Selector) -> Self { value.0 }
}

impl FromLua for Selector {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { Ok(Self(lua.from_value(value)?)) }
}

impl UserData for Selector {}
