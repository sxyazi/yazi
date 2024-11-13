use mlua::{Function, Lua, MultiValue};
use tracing::{debug, error};

use super::Utils;

impl Utils {
	pub(super) fn dbg(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, values: MultiValue| {
			let s = values.into_iter().map(|v| format!("{v:#?}")).collect::<Vec<_>>().join(" ");
			Ok(debug!("{s}"))
		})
	}

	pub(super) fn err(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, values: MultiValue| {
			let s = values.into_iter().map(|v| format!("{v:#?}")).collect::<Vec<_>>().join(" ");
			Ok(error!("{s}"))
		})
	}
}
