use std::collections::HashMap;

use mlua::{ExternalError, Lua, Table, Value, Variadic};
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
				let mut map = HashMap::with_capacity(t.len().map(|l| l as usize)?);
				for result in t.pairs::<Value, Value>() {
					let (k, v) = result?;
					map.insert(Self::value_to_key(k)?, Self::value_to_data(v)?);
				}
				Data::Table(map)
			}
			Value::Function(_) => Err("function is not supported".into_lua_err())?,
			Value::Thread(_) => Err("thread is not supported".into_lua_err())?,
			Value::UserData(_) => Err("userdata is not supported".into_lua_err())?,
			Value::Error(_) => Err("error is not supported".into_lua_err())?,
		})
	}

	pub fn value_to_key(value: Value) -> mlua::Result<DataKey> {
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
			Value::UserData(_) => Err("userdata is not supported".into_lua_err())?,
			Value::Error(_) => Err("error is not supported".into_lua_err())?,
		})
	}

	pub fn data_to_value(lua: &Lua, data: Data) -> mlua::Result<Value> {
		match data {
			Data::Nil => Ok(Value::Nil),
			Data::Boolean(v) => Ok(Value::Boolean(v)),
			Data::Integer(v) => Ok(Value::Integer(v)),
			Data::Number(v) => Ok(Value::Number(v)),
			Data::String(v) => Ok(Value::String(lua.create_string(v)?)),
			Data::Table(v) => {
				let seq_len = v.keys().filter(|&k| !k.is_numeric()).count();
				let table = lua.create_table_with_capacity(seq_len, v.len() - seq_len)?;
				for (k, v) in v {
					table.raw_set(Self::key_to_value(lua, k)?, Self::data_to_value(lua, v)?)?;
				}
				Ok(Value::Table(table))
			}
		}
	}

	pub fn key_to_value(lua: &Lua, key: DataKey) -> mlua::Result<Value> {
		match key {
			DataKey::Nil => Ok(Value::Nil),
			DataKey::Boolean(k) => Ok(Value::Boolean(k)),
			DataKey::Integer(k) => Ok(Value::Integer(k)),
			DataKey::Number(k) => Ok(Value::Number(k.get())),
			DataKey::String(k) => Ok(Value::String(lua.create_string(k)?)),
		}
	}

	pub fn vec_to_table(lua: &Lua, data: Vec<Data>) -> mlua::Result<Table> {
		let mut vec = Vec::with_capacity(data.len());
		for v in data.into_iter() {
			vec.push(Self::data_to_value(lua, v)?);
		}
		lua.create_sequence_from(vec)
	}

	pub fn vec_to_variadic(lua: &Lua, data: Vec<Data>) -> mlua::Result<Variadic<Value>> {
		let mut vec = Vec::with_capacity(data.len());
		for v in data {
			vec.push(Self::data_to_value(lua, v)?);
		}
		Ok(Variadic::from_iter(vec))
	}

	pub fn variadic_to_vec(values: Variadic<Value>) -> mlua::Result<Vec<Data>> {
		let mut vec = Vec::with_capacity(values.len());
		for value in values {
			vec.push(Self::value_to_data(value)?);
		}
		Ok(vec)
	}
}
