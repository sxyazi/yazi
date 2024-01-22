use mlua::{Lua, Table, Value, Variadic};

use super::Utils;

impl Utils {
	pub(super) fn plugin(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"plugin_retrieve",
			lua.create_async_function(
				|_, (_name, _calls, _args): (String, usize, Variadic<Value>)| async move { Ok(()) },
			)?,
		)?;

		Ok(())
	}
}
