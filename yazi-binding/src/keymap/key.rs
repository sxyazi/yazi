use std::ops::Deref;

use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};

use crate::SER_OPT;

#[derive(Clone, Copy, Default, FromLua)]
pub struct Key(pub yazi_config::keymap::Key);

impl Deref for Key {
	type Target = yazi_config::keymap::Key;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl IntoLua for Key {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self.0, SER_OPT) }
}
