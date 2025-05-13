use mlua::{IntoLua, Lua, MetaMethod, Table, Value};

pub struct Composer;

impl Composer {
	pub fn make<F>(lua: &Lua, cap: usize, f: F) -> mlua::Result<Value>
	where
		F: Fn(&Lua, &[u8]) -> mlua::Result<Value> + 'static,
	{
		let index = lua.create_function(move |lua, (ts, key): (Table, mlua::String)| {
			let v = f(lua, &key.as_bytes())?;
			ts.raw_set(key, v.clone())?;
			Ok(v)
		})?;

		let tbl = lua.create_table_with_capacity(0, cap)?;
		tbl.set_metatable(Some(lua.create_table_from([(MetaMethod::Index.name(), index)])?));
		tbl.into_lua(lua)
	}
}
