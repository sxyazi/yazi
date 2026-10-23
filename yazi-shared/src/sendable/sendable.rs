use mlua::{FromLua, IntoLua, Lua, MultiValue};

use crate::data::Data;

pub struct Sendable;

impl Sendable {
	pub fn list_to_values(lua: &Lua, data: Vec<Data>) -> mlua::Result<MultiValue> {
		data.into_iter().map(|d| d.into_lua(lua)).collect()
	}

	pub fn values_to_list(lua: &Lua, values: MultiValue) -> mlua::Result<Vec<Data>> {
		values.into_iter().map(|v| Data::from_lua(v, lua)).collect()
	}
}
