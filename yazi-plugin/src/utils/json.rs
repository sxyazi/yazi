use mlua::{Function, IntoLuaMulti, Lua, LuaSerdeExt, Value};

use super::Utils;
use crate::{Error, config::OPTS};

impl Utils {
	pub(super) fn json_encode(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, value: Value| async move {
			match serde_json::to_string(&value) {
				Ok(s) => (s, Value::Nil).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Serde(e)).into_lua_multi(&lua),
			}
		})
	}

	pub(super) fn json_decode(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, s: mlua::String| async move {
			match serde_json::from_slice::<serde_json::Value>(&s.as_bytes()) {
				Ok(v) => (lua.to_value_with(&v, OPTS)?, Value::Nil).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Serde(e)).into_lua_multi(&lua),
			}
		})
	}
}
