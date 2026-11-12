use std::ffi::OsStr;

use mlua::{BorrowedBytes, ExternalError, Function, IntoLua, IntoLuaMulti, Lua, LuaString, Value};
use yazi_binding::{Composer, ComposerGet};
use yazi_fs::splatter::{SplatSource, Splatter};
use yazi_shim::wtf8::FromWtf8;

use super::Utils;

impl Utils {
	pub(super) fn shell(_: &Lua) -> mlua::Result<Composer<ComposerGet>> {
		fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
			match key {
				b"splat" => Utils::splat(lua)?.into_lua(lua),
				b"split" => Utils::split(lua)?.into_lua(lua),
				_ => Ok(Value::Nil),
			}
		}

		Ok(Composer::new(get))
	}

	pub(super) fn splat(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (cmd, source): (BorrowedBytes, SplatSource)| {
			let s = Splatter::new(source).splat(OsStr::from_wtf8(&cmd)?)?;
			lua.create_external_string(s.into_encoded_bytes())
		})
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
