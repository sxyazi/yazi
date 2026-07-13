use mlua::{ExternalError, FromLua, Function, Lua, LuaString, Value};
use yazi_core::tasks;

use super::Utils;
use crate::tasks::TaskOpt;

impl Utils {
	pub(super) fn task(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (kind, value): (LuaString, Value)| {
			Ok(TaskOpt(match &*kind.as_bytes() {
				b"cut" => tasks::TaskOpt::Cut(<_>::from_lua(value, lua)?),
				b"copy" => tasks::TaskOpt::Copy(<_>::from_lua(value, lua)?),

				b"plugin" => tasks::TaskOpt::Plugin(<_>::from_lua(value, lua)?),

				_ => Err(format!("unsupported spawn kind: {}", kind.display()).into_lua_err())?,
			}))
		})
	}
}
