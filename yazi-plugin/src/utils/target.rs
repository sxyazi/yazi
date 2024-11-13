use mlua::{Function, Lua};

use super::Utils;

impl Utils {
	pub(super) fn target_os(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| Ok(std::env::consts::OS))
	}

	pub(super) fn target_family(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| Ok(std::env::consts::FAMILY))
	}
}
