use mlua::{Lua, Value};

use crate::data::DataAny;

pub struct DataInventory {
	pub from_lua: fn(Value, &Lua) -> mlua::Result<Box<dyn DataAny>>,
}

inventory::collect!(DataInventory);
