use mlua::{Lua, MultiValue, Table};
use tracing::{debug, error};

use super::Utils;

impl Utils {
	pub(super) fn log(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"dbg",
			lua.create_function(|_, values: MultiValue| {
				let s = values.into_iter().map(|v| format!("{v:#?}")).collect::<Vec<_>>().join(" ");
				Ok(debug!("{s}"))
			})?,
		)?;

		ya.raw_set(
			"err",
			lua.create_function(|_, values: MultiValue| {
				let s = values.into_iter().map(|v| format!("{v:#?}")).collect::<Vec<_>>().join(" ");
				Ok(error!("{s}"))
			})?,
		)?;

		Ok(())
	}
}
