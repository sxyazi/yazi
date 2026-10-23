use mlua::{ExternalError, Function, IntoLua, IntoLuaMulti, Lua, LuaString, Value};
use yazi_binding::{Composer, ComposerGet};

use super::Utils;

impl Utils {
	pub(super) fn shell(_: &Lua) -> mlua::Result<Composer<ComposerGet>> {
		fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
			match key {
				b"split" => Utils::split(lua)?.into_lua(lua),
				_ => Ok(Value::Nil),
			}
		}

		Ok(Composer::new(get))
	}

	pub(super) fn split(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, s: LuaString| {
			match yazi_shared::shell::unix::split(&s.to_str()?, false) {
				Ok((words, _)) => words.into_lua_multi(lua),
				Err(e) => (Value::Nil, e.into_lua_err()).into_lua_multi(lua),
			}
		})
	}
}
