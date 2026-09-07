use mlua::{Lua, MultiValue};

use crate::data::Data;

pub struct Sendable;

impl Sendable {
	pub fn list_to_values(lua: &Lua, data: Vec<Data>) -> mlua::Result<MultiValue> {
		data.into_iter().map(|d| Self::data_to_value(lua, d)).collect()
	}

	pub fn values_to_list(lua: &Lua, values: MultiValue) -> mlua::Result<Vec<Data>> {
		values.into_iter().map(|v| Self::value_to_data(lua, v)).collect()
	}
}
