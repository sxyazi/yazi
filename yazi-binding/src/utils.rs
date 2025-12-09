use mlua::{IntoLua, Lua, SerializeOptions, Table, Value};

pub const SER_OPT: SerializeOptions =
	SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false);

pub fn get_metatable(lua: &Lua, value: impl IntoLua) -> mlua::Result<Table> {
	let (_, mt): (Value, Table) = unsafe {
		lua.exec_raw(value.into_lua(lua)?, |state| {
			mlua::ffi::lua_getmetatable(state, -1);
		})
	}?;
	Ok(mt)
}
