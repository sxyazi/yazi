use mlua::{ExternalError, Table, Value};

use crate::toml::DeserializeOverWith;

pub trait DeserializeOverLua: DeserializeOverWith {
	fn deserialize_over_lua(self, table: &Table) -> mlua::Result<Self> {
		let de = mlua::serde::Deserializer::new(Value::Table(table.clone()));
		self.deserialize_over_with(de).map_err(|e| e.into_lua_err())
	}
}

impl<T: DeserializeOverWith> DeserializeOverLua for T {}
