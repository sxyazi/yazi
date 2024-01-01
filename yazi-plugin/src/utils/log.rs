use mlua::{Lua, Table};
use tracing::{debug, error};

use super::Utils;

impl Utils {
	pub(super) fn log(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set("dbg", lua.create_async_function(|_, s: String| async move { Ok(debug!("{s}")) })?)?;

		ya.set("err", lua.create_async_function(|_, s: String| async move { Ok(error!("{s}")) })?)?;

		Ok(())
	}
}
