use mlua::{Lua, Table, Value, Variadic};

use super::Utils;

impl Utils {
	pub(super) fn plugin(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"plugin_retrieve",
			lua.create_async_function(
				|_, (name, calls, args): (String, usize, Variadic<Value>)| async move { Ok(()) },
			)?,
		)?;

		Ok(())
	}
}
