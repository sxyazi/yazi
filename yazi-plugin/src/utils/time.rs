use std::time::{SystemTime, UNIX_EPOCH};

use mlua::{Lua, Table};

use super::Utils;

impl Utils {
	pub(super) fn time(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"time",
			lua.create_function(|_, ()| {
				Ok(SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok())
			})?,
		)?;

		Ok(())
	}
}
