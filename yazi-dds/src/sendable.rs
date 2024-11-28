use std::{borrow::Cow, collections::HashMap};

use mlua::{ExternalError, Lua, MultiValue, Table, Value};
use yazi_shared::{OrderedFloat, event::{Data, DataKey}};

pub struct Sendable;

impl Sendable {
	pub fn value_to_data(value: Value) -> mlua::Result<Data> {
		Ok(match value {
			Value::Nil => Data::Nil,
			Value::Boolean(b) => Data::Boolean(b),
			Value::LightUserData(_) => Err("light userdata is not supported".into_lua_err())?,
			Value::Integer(i) => Data::Integer(i),
			Value::Number(n) => Data::Number(n),
			Value::String(s) => Data::String(s.to_str()?.to_owned()),
			Value::Table(t) => {
				let (mut i, mut map) = (1, HashMap::with_capacity(t.raw_len()));
				for result in t.pairs::<Value, Value>() {
					let (k, v) = result?;
					let k = Self::value_to_key(k)?;

					if k == DataKey::Integer(i) {
						i += 1;
					}
					map.insert(k, Self::value_to_data(v)?);
				}

				if map.len() == i as usize - 1 {
					Data::List((1..i).map(|i| map.remove(&DataKey::Integer(i)).unwrap()).collect())
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
			Value::Other(..) => Err("unknown data is not supported".into_lua_err())?,
		})
	}

	pub fn data_to_value(lua: &Lua, data: Data) -> mlua::Result<Value> {
		Ok(match data {
			Data::List(l) => Value::Table(Self::list_to_table(lua, l)?),
			Data::Dict(d) => Value::Table(Self::dict_to_table(lua, d)?),
			Data::Url(u) => Value::UserData(lua.create_any_userdata(u)?),
			Data::Any(a) => {
				if let Ok(t) = a.downcast::<super::body::BodyYankIter>() {
					Value::UserData(lua.create_userdata(*t)?)
				} else {
					Err("unsupported userdata included".into_lua_err())?
				}
			}
			data => Self::data_to_value_ref(lua, &data)?,
		})
	}

	pub fn data_to_value_ref(lua: &Lua, data: &Data) -> mlua::Result<Value> {
		Ok(match data {
			Data::Nil => Value::Nil,
			Data::Boolean(b) => Value::Boolean(*b),
			Data::Integer(i) => Value::Integer(*i),
			Data::Number(n) => Value::Number(*n),
			Data::String(s) => Value::String(lua.create_string(s)?),
			Data::List(l) => Value::Table(Self::list_to_table_ref(lua, l)?),
			Data::Dict(d) => Value::Table(Self::dict_to_table_ref(lua, d)?),
			Data::Url(u) => Value::UserData(lua.create_any_userdata(u.clone())?),
			Data::Any(a) => {
				if let Some(t) = a.downcast_ref::<super::body::BodyYankIter>() {
					Value::UserData(lua.create_userdata(t.clone())?)
				} else {
					Err("unsupported userdata included".into_lua_err())?
				}
			}
		})
	}

	pub fn list_to_table(lua: &Lua, list: Vec<Data>) -> mlua::Result<Table> {
		let mut vec = Vec::with_capacity(list.len());
		for v in list.into_iter() {
			vec.push(Self::data_to_value(lua, v)?);
		}
		lua.create_sequence_from(vec)
	}

	pub fn list_to_table_ref(lua: &Lua, list: &[Data]) -> mlua::Result<Table> {
		let mut vec = Vec::with_capacity(list.len());
		for v in list {
			vec.push(Self::data_to_value_ref(lua, v)?);
		}
		lua.create_sequence_from(vec)
	}

	pub fn dict_to_table(lua: &Lua, dict: HashMap<DataKey, Data>) -> mlua::Result<Table> {
		let seq_len = dict.keys().filter(|&k| !k.is_integer()).count();
		let tbl = lua.create_table_with_capacity(seq_len, dict.len() - seq_len)?;
		for (k, v) in dict {
			tbl.raw_set(Self::key_to_value(lua, k)?, Self::data_to_value(lua, v)?)?;
		}
		Ok(tbl)
	}

	pub fn dict_to_table_ref(lua: &Lua, dict: &HashMap<DataKey, Data>) -> mlua::Result<Table> {
		let seq_len = dict.keys().filter(|&k| !k.is_integer()).count();
		let tbl = lua.create_table_with_capacity(seq_len, dict.len() - seq_len)?;
		for (k, v) in dict {
			tbl.raw_set(Self::key_to_value_ref(lua, k)?, Self::data_to_value_ref(lua, v)?)?;
		}
		Ok(tbl)
	}

	pub fn list_to_values(lua: &Lua, data: Vec<Data>) -> mlua::Result<MultiValue> {
		let mut vec = Vec::with_capacity(data.len());
		for v in data {
			vec.push(Self::data_to_value(lua, v)?);
		}
		Ok(MultiValue::from_vec(vec))
	}

	pub fn values_to_list(values: MultiValue) -> mlua::Result<Vec<Data>> {
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
			Value::String(v) => DataKey::String(Cow::Owned(v.to_str()?.to_owned())),
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
			Value::Other(..) => Err("unknown data is not supported".into_lua_err())?,
		})
	}

	fn key_to_value(lua: &Lua, key: DataKey) -> mlua::Result<Value> {
		Ok(match key {
			DataKey::Url(u) => Value::UserData(lua.create_any_userdata(u)?),
			key => Self::key_to_value_ref(lua, &key)?,
		})
	}

	fn key_to_value_ref(lua: &Lua, key: &DataKey) -> mlua::Result<Value> {
		Ok(match key {
			DataKey::Nil => Value::Nil,
			DataKey::Boolean(b) => Value::Boolean(*b),
			DataKey::Integer(i) => Value::Integer(*i),
			DataKey::Number(n) => Value::Number(n.get()),
			DataKey::String(s) => Value::String(lua.create_string(s.as_ref())?),
			DataKey::Url(u) => Value::UserData(lua.create_any_userdata(u.clone())?),
		})
	}
}
