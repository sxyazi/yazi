use mlua::{Lua, Table};

use super::Utils;

impl Utils {
	pub(super) fn target(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"target_family",
			lua.create_function(|_, ()| {
				#[cfg(unix)]
				{
					Ok("unix")
				}
				#[cfg(windows)]
				{
					Ok("windows")
				}
				#[cfg(wasm)]
				{
					Ok("wasm")
				}
			})?,
		)?;

		Ok(())
	}
}
