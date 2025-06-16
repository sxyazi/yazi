use mlua::{Function, IntoLua, IntoLuaMulti, Lua, Value};
use yazi_adapter::{Dimension, EMULATOR};

use crate::Composer;

pub(super) struct Term;

impl Term {
	pub(super) fn compose(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, |lua, key| match key {
			b"light" => EMULATOR.get().light.into_lua(lua),
			b"cell_size" => Self::cell_size(lua)?.into_lua(lua),
			_ => Ok(Value::Nil),
		})
	}

	fn cell_size(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, ()| {
			if let Some(s) = Dimension::cell_size() {
				s.into_lua_multi(lua)
			} else {
				().into_lua_multi(lua)
			}
		})
	}
}
