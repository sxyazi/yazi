use std::collections::HashMap;

use mlua::{ExternalError, Lua, MultiValue, Table, Value};
use yazi_shared::{event::{Data, DataKey}, OrderedFloat};

pub struct Sendable;

impl Sendable {
	pub fn value_to_data(value: Value) -> mlua::Result<Data> {
		Ok(match value {
			Value::Nil => Data::Nil,
			Value::Boolean(b) => Data::Boolean(b),
			Value::LightUserData(_) => Err("light userdata is not supported".into_lua_err())?,
			Value::Integer(n) => Data::Integer(n),
			Value::Number(n) => Data::Number(n),
			Value::String(s) => Data::String(s.to_str()?.to_owned()),
			Value::Table(t) => {
				let (mut i, mut map) = (0, HashMap::with_capacity(t.raw_len()));
				for result in t.pairs::<Value, Value>() {
					let (k, v) = result?;
					let k = Self::value_to_key(k)?;

					if k == DataKey::Integer(i) {
						i += 1;
					}
					map.insert(k, Self::value_to_data(v)?);
				}

				if i as usize == map.len() {
					Data::List(map.into_values().collect())
				} else {
					Data::Dict(map)
				}
			}
			Value::Function(_) => Err("function is not supported".into_lua_err())?,
			Value::Thread(_) => Err("thread is not supported".into_lua_err())?,
			Value::UserData(ud) => {
				if let Ok(t) = ud.take::<yazi_shared::fs::Url>() {
					Data::Url(t)
				} else if let Ok(t) = ud.take::<super::body::BodyYankIter>() {
					Data::Any(Box::new(t))
				} else {
					Err("unsupported userdata included".into_lua_err())?
				}
			}
			Value::Error(_) => Err("error is not supported".into_lua_err())?,
		})
	}

	pub fn data_to_value(lua: &Lua, data: Data) -> mlua::Result<Value> {
		Ok(match data {
			Data::Nil => Value::Nil,
			Data::Boolean(v) => Value::Boolean(v),
			Data::Integer(v) => Value::Integer(v),
			Data::Number(v) => Value::Number(v),
			Data::String(v) => Value::String(lua.create_string(v)?),
			Data::List(v) => Value::Table(Self::list_to_table(lua, v)?),
			Data::Dict(t) => {
				let seq_len = t.keys().filter(|&k| !k.is_integer()).count();
				let table = lua.create_table_with_capacity(seq_len, t.len() - seq_len)?;
				for (k, v) in t {
					table.raw_set(Self::key_to_value(lua, k)?, Self::data_to_value(lua, v)?)?;
				}
				Value::Table(table)
			}
			Data::Url(v) => Value::UserData(lua.create_any_userdata(v)?),
			Data::Any(v) => {
				if let Ok(t) = v.downcast::<super::body::BodyYankIter>() {
					Value::UserData(lua.create_userdata(*t)?)
				} else {
					Err("unsupported userdata included".into_lua_err())?
				}
			}
		})
	}

	pub fn list_to_table(lua: &Lua, data: Vec<Data>) -> mlua::Result<Table> {
		let mut vec = Vec::with_capacity(data.len());
		for v in data.into_iter() {
			vec.push(Self::data_to_value(lua, v)?);
		}
		lua.create_sequence_from(vec)
	}

	pub fn list_to_values(lua: &Lua, data: Vec<Data>) -> mlua::Result<MultiValue> {
		let mut vec = Vec::with_capacity(data.len());
		for v in data {
			vec.push(Self::data_to_value(lua, v)?);
		}
		Ok(MultiValue::from_iter(vec))
	}

	pub fn values_to_vec(values: MultiValue) -> mlua::Result<Vec<Data>> {
		let mut vec = Vec::with_capacity(values.len());
		for value in values {
			vec.push(Self::value_to_data(value)?);
		}
		Ok(vec)
	}

	fn value_to_key(value: Value) -> mlua::Result<DataKey> {
		Ok(match value {
			Value::Nil => DataKey::Nil,
			Value::Boolean(v) => DataKey::Boolean(v),
			Value::LightUserData(_) => Err("light userdata is not supported".into_lua_err())?,
			Value::Integer(v) => DataKey::Integer(v),
			Value::Number(v) => DataKey::Number(OrderedFloat::new(v)),
			Value::String(v) => DataKey::String(v.to_str()?.to_owned()),
			Value::Table(_) => Err("table is not supported".into_lua_err())?,
			Value::Function(_) => Err("function is not supported".into_lua_err())?,
			Value::Thread(_) => Err("thread is not supported".into_lua_err())?,
			Value::UserData(ud) => {
				if let Ok(t) = ud.take::<yazi_shared::fs::Url>() {
					DataKey::Url(t)
				} else {
					Err("unsupported userdata included".into_lua_err())?
				}
			}
			Value::Error(_) => Err("error is not supported".into_lua_err())?,
		})
	}

	fn key_to_value(lua: &Lua, key: DataKey) -> mlua::Result<Value> {
		Ok(match key {
			DataKey::Nil => Value::Nil,
			DataKey::Boolean(k) => Value::Boolean(k),
			DataKey::Integer(k) => Value::Integer(k),
			DataKey::Number(k) => Value::Number(k.get()),
			DataKey::String(k) => Value::String(lua.create_string(k)?),
			DataKey::Url(k) => Value::UserData(lua.create_any_userdata(k)?),
		})
	}
}
