use mlua::{Function, IntoLuaMulti, Lua, LuaSerdeExt, MetaMethod, Result, Table, Value};
use yazi_binding::Error;

use std::cell::RefCell;
use std::rc::Rc;

use super::Utils;
use crate::config::OPTS;

struct OrderedTable<'lua> {
	metatable: Table,
	lua: &'lua Lua,
}

impl<'lua> OrderedTable<'lua> {
	const DATA_KEY: &'static str = "__data";
	const ORDERED_KEY: &'static str = "__ordered";

	fn new(lua: &'lua Lua) -> Result<Self> {
		let mt = lua.create_table()?;

		mt.set(
			MetaMethod::Index.name(),
			lua.create_function(|_, (table, key): (Table, Value)| {
				let data: Table = table.get(Self::DATA_KEY)?;
				data.get::<Value>(key)
			})?,
		)?;

		mt.set(
			MetaMethod::Pairs.name(),
			lua.create_function(|lua, table: Table| {
				let ordered: Value = table.get(Self::ORDERED_KEY)?;
				let data: Table = table.get(Self::DATA_KEY)?;

				let idx = Rc::new(RefCell::new(0));

				let iter = lua.create_function_mut(move |_, ()| {
					let mut index = idx.borrow_mut();

					*index += 1;

					let key: Value = match &ordered {
						Value::Table(ordered) => ordered.get(*index)?,
						_ => {
							if data.len()? >= *index {
								Value::Integer(*index)
							} else {
								Value::Nil
							}
						}
					};

					if key == Value::Nil {
						Ok((Value::Nil, Value::Nil))
					} else {
						let val = data.get(key.clone())?;
						Ok((key, val))
					}
				})?;

				Ok((iter, Value::Nil, Value::Nil))
			})?,
		)?;

		mt.set(
			MetaMethod::Len.name(),
			lua.create_function(|_, table: Table| {
				let data: Table = table.get(Self::DATA_KEY)?;

				data.len()
			})?,
		)?;

		Ok(Self { metatable: mt, lua })
	}

	fn wrap(&self, value: &serde_json::Value) -> Result<Value> {
		let data_table = self.lua.create_table()?;
		let ordered_table = self.lua.create_table()?;
		let wrapper = self.lua.create_table()?;

		wrapper.set_metatable(Some(self.metatable.clone()));

		match &value {
			serde_json::Value::Object(obj) => {
				let mut idx = 1;

				for (k, v) in obj {
					let key_str = self.lua.create_string(k)?;
					let val = self.wrap(v)?;

					data_table.set(key_str.clone(), val)?;
					ordered_table.set(idx, key_str)?;
					idx += 1;
				}

				wrapper.set(Self::DATA_KEY, data_table)?;
				wrapper.set(Self::ORDERED_KEY, ordered_table)?;

				Ok(Value::Table(wrapper))
			}
			serde_json::Value::Array(arr) => {
				for (i, v) in arr.iter().enumerate() {
					data_table.set(i + 1, self.wrap(v)?)?;
				}

				wrapper.set(Self::DATA_KEY, data_table)?;
				wrapper.set(Self::ORDERED_KEY, Value::Nil)?;

				Ok(Value::Table(wrapper))
			}
			_ => self.lua.to_value_with(value, OPTS),
		}
	}
}

impl Utils {
	pub(super) fn json_encode(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, value: Value| async move {
			match serde_json::to_string(&value) {
				Ok(s) => (s, Value::Nil).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Serde(e)).into_lua_multi(&lua),
			}
		})
	}

	pub(super) fn json_decode(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (s, ordered): (mlua::String, Option<Value>)| async move {
			let ordered = ordered.unwrap_or(Value::Boolean(false));

			match serde_json::from_slice::<serde_json::Value>(&s.as_bytes()) {
				Ok(v) => (
					if let Value::Boolean(true) = ordered {
						let ordered_table = OrderedTable::new(&lua)?;
						ordered_table.wrap(&v)?
					} else {
						lua.to_value_with(&v, OPTS)?
					},
					Value::Nil,
				)
					.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Serde(e)).into_lua_multi(&lua),
			}
		})
	}
}
