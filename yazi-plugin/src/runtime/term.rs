use mlua::{Function, IntoLua, IntoLuaMulti, Lua, Value};
use yazi_binding::{Composer, ComposerGet};
use yazi_emulator::{Dimension, EMULATOR};

pub(super) fn term() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"light" => light(lua)?.into_lua(lua),
			b"cell_size" => cell_size(lua)?.into_lua(lua),
			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn light(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, ()| EMULATOR.light().into_lua(lua))
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
