use mlua::{
	Function, IntoLuaMulti, Lua, LuaSerdeExt, MetaMethod, UserData, UserDataMethods, Value,
};

use serde::Serialize;
use serde_json::value::Value as JsonValue;

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use yazi_binding::Error;

use super::Utils;
use crate::config::OPTS;

#[derive(Clone)]
enum OrderedTableIndex {
	Key(String),
	Index(usize),
}

#[derive(Clone)]
struct OrderedTable {
	data: Rc<RefCell<JsonValue>>,
	path: Vec<OrderedTableIndex>,
}

impl Serialize for OrderedTable {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use OrderedTableIndex::*;
		let current = self.data.borrow();

		let mut value = &*current;
		for index in &self.path {
			match index {
				Key(key) => {
					value = value
						.get(key)
						.ok_or_else(|| serde::ser::Error::custom(format!("Key '{}' not found", key)))?;
				}
				Index(i) => {
					value = value
						.get(*i)
						.ok_or_else(|| serde::ser::Error::custom(format!("Index {} out of bounds", i)))?;
				}
			}
		}

		value.serialize(serializer)
	}
}

impl OrderedTable {
	fn new(data: Rc<RefCell<JsonValue>>, path: Vec<OrderedTableIndex>) -> Self {
		Self { data, path }
	}

	fn get_current(&self) -> Option<Ref<JsonValue>> {
		let root = self.data.borrow();

		let mut tmp = &*root;
		for comp in &self.path {
			tmp = match comp {
				OrderedTableIndex::Key(k) => tmp.get(k)?,
				OrderedTableIndex::Index(i) => tmp.get(*i)?,
			};
		}

		Some(Ref::map(root, move |root_value| {
			let mut current = root_value;
			for comp in &self.path {
				current = match comp {
					OrderedTableIndex::Key(k) => current.get(k).unwrap(),
					OrderedTableIndex::Index(i) => current.get(*i).unwrap(),
				};
			}
			current
		}))
	}

	fn get_current_mut(&self) -> Option<RefMut<JsonValue>> {
		let root = self.data.borrow_mut();

		let mut tmp = &*root;
		for comp in &self.path {
			tmp = match comp {
				OrderedTableIndex::Key(k) => tmp.get(k)?,
				OrderedTableIndex::Index(i) => tmp.get(*i)?,
			};
		}

		Some(RefMut::map(root, move |root_value| {
			let mut current = root_value;
			for comp in &self.path {
				current = match comp {
					OrderedTableIndex::Key(k) => current.get_mut(k).unwrap(),
					OrderedTableIndex::Index(i) => current.get_mut(*i).unwrap(),
				};
			}
			current
		}))
	}
}

impl UserData for OrderedTable {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_meta_field("__ordered", true);
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |lua, this: &OrderedTable, key: Value| {
			let new_path = match key {
				Value::String(s) => OrderedTableIndex::Key(s.to_str()?.to_string()),
				Value::Integer(i) => OrderedTableIndex::Index((i - 1) as usize),
				_ => return Ok(Value::Nil),
			};

			let sub = OrderedTable::new(Rc::clone(&this.data), {
				let mut p = this.path.clone();
				p.push(new_path);
				p
			});

			if let Some(v) = sub.clone().get_current() {
				if matches!(*v, JsonValue::Object(_) | JsonValue::Array(_)) {
					Ok(Value::UserData(lua.create_userdata(sub)?))
				} else {
					lua.to_value_with(&*v, OPTS)
				}
			} else {
				Ok(Value::Nil)
			}
		});

		methods.add_meta_method_mut(MetaMethod::Pairs, |lua, this: &mut OrderedTable, ()| {
			let curr =
				this.get_current_mut().ok_or_else(|| mlua::Error::RuntimeError("invalid path".into()))?;

			let keys = match &*curr {
				JsonValue::Object(obj) => {
					obj.keys().map(|k| (Value::String(lua.create_string(k).unwrap()))).collect()
				}
				JsonValue::Array(arry) => (1..=arry.len()).map(|i| Value::Integer(i as i64)).collect(),
				_ => vec![],
			};
			let path = this.path.clone();
			let data = Rc::clone(&this.data);
			let idx = Rc::new(RefCell::new(keys.into_iter()));
			let iter = lua.create_function_mut(move |lua, ()| {
				let mut index = idx.borrow_mut();
				if let Some(key) = index.next() {
					let new_path = match &key {
						Value::String(s) => OrderedTableIndex::Key(s.to_str()?.to_string()),
						Value::Integer(i) => OrderedTableIndex::Index((i - 1) as usize),
						_ => return Ok((Value::Nil, Value::Nil)),
					};

					Ok((
						key.clone(),
						Value::UserData(lua.create_userdata(OrderedTable::new(Rc::clone(&data), {
							let mut p = path.clone();
							p.push(new_path);
							p
						}))?),
					))
				} else {
					Ok((Value::Nil, Value::Nil))
				}
			})?;

			Ok((iter, Value::Nil, Value::Nil))
		});

		methods.add_meta_method(MetaMethod::Len, |_, this: &OrderedTable, ()| {
			Ok(if let Some(data) = this.get_current() {
				Value::Integer(match &*data {
					JsonValue::Array(arr) => arr.len() as i64,
					_ => 0,
				})
			} else {
				Value::Nil
			})
		});

		methods.add_meta_method_mut(
			MetaMethod::NewIndex,
			|_, this: &mut OrderedTable, (key, value): (Value, Value)| {
				let value = serde_json::to_value(value).map_err(mlua::Error::external)?;
				let mut curr =
					this.get_current_mut().ok_or_else(|| mlua::Error::RuntimeError("invalid path".into()))?;

				match &mut *curr {
					JsonValue::Object(obj) => {
						let k = key
							.as_str()
							.ok_or_else(|| mlua::Error::RuntimeError("object key must be string".into()))?;
						obj.insert(k.to_string(), value);
					}
					JsonValue::Array(arr) => {
						let i = key
							.as_integer()
							.ok_or_else(|| mlua::Error::RuntimeError("array index must be integer".into()))?;
						let idx = (i - 1) as usize;
						if idx < arr.len() {
							arr[idx] = value;
						}
					}
					_ => {
						return Err(mlua::Error::RuntimeError("not an object or array".into()));
					}
				}

				Ok(())
			},
		);
	}
}

impl Utils {
	pub(super) fn json_encode(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, value: Value| async move {
			let result = match value {
				Value::UserData(ud) => {
					let table = ud
						.borrow::<OrderedTable>()
						.map_err(|_| mlua::Error::RuntimeError("Unknown userdata type".into()))?;
					serde_json::to_string(&*table).map_err(Error::Serde)
				}
				_ => serde_json::to_string(&value).map_err(Error::Serde),
			};

			match result {
				Ok(s) => Ok((s, Value::Nil).into_lua_multi(&lua)?),
				Err(e) => Ok((Value::Nil, e).into_lua_multi(&lua)?),
			}
		})
	}

	pub(super) fn json_decode(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (s, ordered): (mlua::String, Option<Value>)| async move {
			let ordered = ordered.unwrap_or(Value::Boolean(false));

			match serde_json::from_slice::<JsonValue>(&s.as_bytes()) {
				Ok(v) => (
					if let Value::Boolean(true) = ordered {
						let data = Rc::new(RefCell::new(v));

						Value::UserData(lua.create_userdata(OrderedTable::new(data, vec![]))?)
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
