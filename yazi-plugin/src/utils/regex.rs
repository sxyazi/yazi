use std::borrow::Cow;

use mlua::{ExternalError, Function, IntoLua, IntoLuaMulti, Lua, LuaString, Value};
use yazi_binding::{Composer, ComposerGet};
use yazi_fs::Normalizer;

use super::Utils;

impl Utils {
	pub(super) fn regex(_: &Lua) -> mlua::Result<Composer<ComposerGet>> {
		fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
			match key {
				b"normalize" => Utils::normalize(lua)?.into_lua(lua),
				_ => Ok(Value::Nil),
			}
		}

		Ok(Composer::new(get))
	}

	fn normalize(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, s: LuaString| match Normalizer::normalize(&s.to_str()?) {
			Ok(Cow::Borrowed(_)) => s.into_lua_multi(lua),
			Ok(Cow::Owned(s)) => lua.create_external_string(s)?.into_lua_multi(lua),
			Err(e) => (Value::Nil, e.into_lua_err()).into_lua_multi(lua),
		})
	}
}
