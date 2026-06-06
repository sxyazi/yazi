use std::marker::PhantomData;

use mlua::{ExternalError, FromLua, Lua, Table, Value};

use crate::{mlua::SequenceIter, toml::DeserializeOverWith};

// --- DeserializeOverLua
pub trait DeserializeOverLua: DeserializeOverWith {
	fn deserialize_over_lua(self, table: &Table) -> mlua::Result<Self> {
		let de = mlua::serde::Deserializer::new(Value::Table(table.clone()));
		self.deserialize_over_with(de).map_err(|e| e.into_lua_err())
	}
}

impl<T: DeserializeOverWith> DeserializeOverLua for T {}

// --- LuaTableExt
pub trait LuaTableExt {
	fn sequence_iter<V: FromLua>(&self, lua: &Lua) -> SequenceIter<V>;
}

impl LuaTableExt for Table {
	fn sequence_iter<V: FromLua>(&self, lua: &Lua) -> SequenceIter<V> {
		SequenceIter {
			lua:      lua.clone(),
			table:    self.clone(),
			index:    0,
			_phantom: PhantomData,
		}
	}
}
