use mlua::{Function, IntoLua, IntoLuaMulti, Lua, Value};
use yazi_adapter::{Dimension, EMULATOR};
use yazi_binding::{Composer, ComposerGet, ComposerSet};

pub(super) fn term() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"light" => EMULATOR.light.into_lua(lua),
			b"cell_size" => cell_size(lua)?.into_lua(lua),
			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
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
