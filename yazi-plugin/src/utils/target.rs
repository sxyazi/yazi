use mlua::{Lua, Table};

use super::Utils;

impl Utils {
	pub(super) fn target(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set("target_os", lua.create_function(|_, ()| Ok(std::env::consts::OS))?)?;
		ya.raw_set("target_family", lua.create_function(|_, ()| Ok(std::env::consts::FAMILY))?)?;

		Ok(())
	}
}
