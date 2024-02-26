use mlua::{Lua, Table};
use tracing::{debug, error};

use super::Utils;

impl Utils {
	pub(super) fn log(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set("dbg", lua.create_function(|_, s: String| Ok(debug!("{s}")))?)?;

		ya.raw_set("err", lua.create_function(|_, s: String| Ok(error!("{s}")))?)?;

		Ok(())
	}
}
