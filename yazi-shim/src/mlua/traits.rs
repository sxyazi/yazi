use std::marker::PhantomData;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, UserData, UserDataFields, Value};

use crate::{mlua::{SequenceIter, is_alive}, toml::DeserializeOverWith};

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

// --- UserDataFieldsExt
pub trait UserDataFieldsExt<S: UserData + 'static>: UserDataFields<S> {
	fn add_cached_field<R: IntoLua + 'static>(
		&mut self,
		key: &'static str,
		compute: fn(&Lua, &S) -> mlua::Result<R>,
	) -> &mut Self {
		self.add_field_function_get(key, move |lua, ud| {
			match ud.named_user_value::<Option<Value>>(key)? {
				Some(v) if is_alive(&v) => Ok(v),
				_ => {
					let v = compute(lua, &*ud.borrow::<S>()?)?.into_lua(lua)?;
					ud.set_named_user_value(key, &v)?;
					Ok(v)
				}
			}
		});
		self
	}

	fn add_cached_field_mut<R: IntoLua + 'static>(
		&mut self,
		key: &'static str,
		compute: fn(&Lua, &mut S) -> mlua::Result<R>,
	) -> &mut Self {
		self.add_field_function_get(key, move |lua, ud| {
			match ud.named_user_value::<Option<Value>>(key)? {
				Some(Value::Error(e)) => Err(*e),
				Some(v) if is_alive(&v) => Ok(v),
				_ => {
					let v = match compute(lua, &mut *ud.borrow_mut::<S>()?).and_then(|r| r.into_lua(lua)) {
						Ok(v) => v,
						Err(e) => Value::Error(Box::new(e)),
					};

					ud.set_named_user_value(key, &v)?;
					Ok(v)
				}
			}
		});
		self
	}

	fn add_static_field<R: IntoLua + 'static>(
		&mut self,
		key: &'static str,
		compute: fn(&Lua, &S) -> mlua::Result<R>,
	) -> &mut Self {
		self.add_field_function_get(key, move |lua, ud| {
			match ud.named_user_value::<Option<Value>>(key)? {
				Some(v) => Ok(v),
				_ => {
					let v = compute(lua, &*ud.borrow::<S>()?)?.into_lua(lua)?;
					ud.set_named_user_value(key, &v)?;
					Ok(v)
				}
			}
		});
		self
	}
}

impl<S: UserData + 'static, F: UserDataFields<S>> UserDataFieldsExt<S> for F {}
