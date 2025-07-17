use mlua::{IntoLua, Lua, Table, Value};

pub fn get_metatable(lua: &Lua, value: impl IntoLua) -> mlua::Result<Table> {
	let (_, mt): (Value, Table) = unsafe {
		lua.exec_raw(value.into_lua(lua)?, |state| {
			mlua::ffi::lua_getmetatable(state, -1);
		})
	}?;
	Ok(mt)
}
