#![allow(clippy::module_inception)]

use mlua::{IntoLua, Lua, MetaMethod, Table, Value};

yazi_macro::mod_flat!(pubsub);

pub(super) fn compose(lua: &Lua) -> mlua::Result<Table> {
	let index = lua.create_function(|lua, (ts, key): (Table, mlua::String)| {
		let value = match key.as_bytes().as_ref() {
			b"pub" => Pubsub::pub_(lua)?,
			b"pub_to" => Pubsub::pub_to(lua)?,
			b"sub" => Pubsub::sub(lua)?,
			b"sub_remote" => Pubsub::sub_remote(lua)?,
			b"unsub" => Pubsub::unsub(lua)?,
			b"unsub_remote" => Pubsub::unsub_remote(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)?;

		ts.raw_set(key, value.clone())?;
		Ok(value)
	})?;

	let ps = lua.create_table_with_capacity(0, 10)?;
	ps.set_metatable(Some(lua.create_table_from([(MetaMethod::Index.name(), index)])?));

	Ok(ps)
}
