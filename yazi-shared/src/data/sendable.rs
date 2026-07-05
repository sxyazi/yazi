use std::{any::TypeId, borrow::Cow};

use hashbrown::HashMap;
use mlua::{ExternalError, IntoLua, Lua, MultiValue, Table, Value};
use ordered_float::OrderedFloat;

use crate::{any_data::AnyData, data::{Data, DataInventory, DataKey}, id::Id, path::PathBufDyn, replace_cow, url::UrlBuf};

pub struct Sendable;

impl Sendable {
	pub fn value_to_data(lua: &Lua, value: Value) -> mlua::Result<Data> {
		match &value {
			Value::Nil => return Ok(Data::Nil),
			Value::Boolean(b) => return Ok(Data::Boolean(*b)),
			Value::Integer(i) => return Ok(Data::Integer(*i)),
			Value::Number(n) => return Ok(Data::Number(*n)),
			Value::String(b) => {
				return Ok(if let Ok(s) = b.to_str() {
					Data::String(s.to_owned().into())
				} else {
					Data::Bytes(b.as_bytes().to_owned())
				});
			}
			Value::Table(t) => {
				let (mut i, mut map) = (1, HashMap::with_capacity(t.raw_len()));
				for result in t.pairs::<Value, Value>() {
					let (k, v) = result?;
					let k = Self::value_to_key(k)?;

					if k == DataKey::Integer(i) {
						i += 1;
					}
					map.insert(k, Self::value_to_data(lua, v)?);
				}

				return Ok(if map.len() == i as usize - 1 {
					Data::List((1..i).map(|i| map.remove(&DataKey::Integer(i)).unwrap()).collect())
				} else {
					Data::Dict(map)
				});
			}
			Value::UserData(ud) => match ud.type_id() {
				Some(t) if t == TypeId::of::<UrlBuf>() => {
					return Ok(Data::Url(ud.take()?));
				}
				Some(t) if t == TypeId::of::<PathBufDyn>() => {
					return Ok(Data::Path(ud.take()?));
				}
				Some(t) if t == TypeId::of::<Id>() => {
					return Ok(Data::Id(*ud.borrow::<Id>()?));
				}
				Some(t) if t == TypeId::of::<AnyData>() => return Ok(Data::Any(ud.take::<AnyData>()?.0)),
				_ => {}
			},
			Value::LightUserData(_) => {}
			Value::Function(_) => {}
			Value::Thread(_) => {}
			Value::Error(_) => {}
			Value::Other(_) => {}
		}

		for inv in inventory::iter::<DataInventory> {
			match (inv.from_lua)(&value, lua) {
				Ok(data) => return Ok(Data::Any(data)),
				Err(mlua::Error::UserDataTypeMismatch) => continue,
				Err(e) => return Err(e),
			}
		}
		Err(format!("unsupported value included: {value:?}").into_lua_err())?
	}

	pub fn data_to_value(lua: &Lua, data: Data) -> mlua::Result<Value> {
		Ok(match data {
			Data::String(Cow::Owned(s)) => Value::String(lua.create_external_string(s)?),
			Data::List(l) => {
				let mut vec = Vec::with_capacity(l.len());
				for v in l.into_iter() {
					vec.push(Self::data_to_value(lua, v)?);
				}
				Value::Table(lua.create_sequence_from(vec)?)
			}
			Data::Dict(d) => {
				let seq_len = d.keys().filter(|&k| k.is_integer()).count();
				let tbl = lua.create_table_with_capacity(seq_len, d.len() - seq_len)?;
				for (k, v) in d {
					tbl.raw_set(Self::key_to_value(lua, k)?, Self::data_to_value(lua, v)?)?;
				}
				Value::Table(tbl)
			}
			Data::Url(u) => u.into_lua(lua)?,
			Data::Path(p) => p.into_lua(lua)?,
			Data::Bytes(b) => Value::String(lua.create_external_string(b)?),
			Data::Any(a) => a.into_lua(lua)?,
			_ => Self::data_to_value_ref(lua, &data)?,
		})
	}

	pub fn data_to_value_ref(lua: &Lua, data: &Data) -> mlua::Result<Value> {
		Ok(match data {
			Data::Nil => Value::Nil,
			Data::Boolean(b) => Value::Boolean(*b),
			Data::Integer(i) => Value::Integer(*i),
			Data::Number(n) => Value::Number(*n),
			Data::String(s) => Value::String(lua.create_string(&**s)?),
			Data::List(l) => {
				let mut vec = Vec::with_capacity(l.len());
				for v in l {
					vec.push(Self::data_to_value_ref(lua, v)?);
				}
				Value::Table(lua.create_sequence_from(vec)?)
			}
			Data::Dict(d) => {
				let seq_len = d.keys().filter(|&k| k.is_integer()).count();
				let tbl = lua.create_table_with_capacity(seq_len, d.len() - seq_len)?;
				for (k, v) in d {
					tbl.raw_set(Self::key_to_value_ref(lua, k)?, Self::data_to_value_ref(lua, v)?)?;
				}
				Value::Table(tbl)
			}
			Data::Id(i) => i.into_lua(lua)?,
			Data::Url(u) => u.clone().into_lua(lua)?,
			Data::Path(p) => p.clone().into_lua(lua)?,
			Data::Bytes(b) => Value::String(lua.create_string(b)?),
			Data::Any(a) => a.to_lua(lua)?,
		})
	}

	pub fn table_to_args(lua: &Lua, t: Table) -> mlua::Result<HashMap<DataKey, Data>> {
		let mut args = HashMap::with_capacity(t.raw_len());
		for pair in t.pairs::<Value, Value>() {
			let (k, v) = pair?;
			match k {
				Value::Integer(i) if i > 0 => {
					args.insert(DataKey::Integer(i - 1), Self::value_to_data(lua, v)?);
				}
				Value::String(s) => {
					args.insert(
						DataKey::String(Cow::Owned(s.to_str()?.replace('_', "-"))),
						Self::value_to_data(lua, v)?,
					);
				}
				_ => return Err("invalid key in Action".into_lua_err()),
			}
		}
		Ok(args)
	}

	pub fn args_to_table(lua: &Lua, args: HashMap<DataKey, Data>) -> mlua::Result<Table> {
		let seq_len = args.keys().filter(|&k| k.is_integer()).count();
		let tbl = lua.create_table_with_capacity(seq_len, args.len() - seq_len)?;
		for (k, v) in args {
			match k {
				DataKey::Integer(i) => tbl.raw_set(i + 1, Self::data_to_value(lua, v)?),
				DataKey::String(s) => tbl.raw_set(replace_cow(s, "-", "_"), Self::data_to_value(lua, v)?),
				_ => Err("invalid key in Data".into_lua_err()),
			}?;
		}
		Ok(tbl)
	}

	pub fn args_to_table_ref(lua: &Lua, args: &HashMap<DataKey, Data>) -> mlua::Result<Table> {
		let seq_len = args.keys().filter(|&k| k.is_integer()).count();
		let tbl = lua.create_table_with_capacity(seq_len, args.len() - seq_len)?;
		for (k, v) in args {
			match k {
				DataKey::Integer(i) => tbl.raw_set(i + 1, Self::data_to_value_ref(lua, v)?),
				DataKey::String(s) => {
					tbl.raw_set(replace_cow(&**s, "-", "_"), Self::data_to_value_ref(lua, v)?)
				}
				_ => Err("invalid key in Data".into_lua_err()),
			}?;
		}
		Ok(tbl)
	}

	pub fn list_to_values(lua: &Lua, data: Vec<Data>) -> mlua::Result<MultiValue> {
		data.into_iter().map(|d| Self::data_to_value(lua, d)).collect()
	}

	pub fn values_to_list(lua: &Lua, values: MultiValue) -> mlua::Result<Vec<Data>> {
		values.into_iter().map(|v| Self::value_to_data(lua, v)).collect()
	}
}

impl Sendable {
	fn value_to_key(value: Value) -> mlua::Result<DataKey> {
		Ok(match value {
			Value::Nil => DataKey::Nil,
			Value::Boolean(b) => DataKey::Boolean(b),
			Value::LightUserData(_) => Err("light userdata is not supported".into_lua_err())?,
			Value::Integer(i) => DataKey::Integer(i),
			Value::Number(n) => DataKey::Number(OrderedFloat(n)),
			Value::String(s) => {
				if let Ok(s) = s.to_str() {
					DataKey::String(s.to_owned().into())
				} else {
					DataKey::Bytes(s.as_bytes().to_owned())
				}
			}
			Value::Table(_) => Err("table is not supported".into_lua_err())?,
			Value::Function(_) => Err("function is not supported".into_lua_err())?,
			Value::Thread(_) => Err("thread is not supported".into_lua_err())?,
			Value::UserData(ud) => match ud.type_id() {
				Some(t) if t == TypeId::of::<UrlBuf>() => DataKey::Url(ud.take()?),
				Some(t) if t == TypeId::of::<PathBufDyn>() => DataKey::Path(ud.take()?),
				Some(t) if t == TypeId::of::<Id>() => DataKey::Id(*ud.borrow::<Id>()?),
				_ => Err(format!("unsupported userdata included: {ud:?}").into_lua_err())?,
			},
			Value::Error(_) => Err("error is not supported".into_lua_err())?,
			Value::Other(..) => Err("unknown data is not supported".into_lua_err())?,
		})
	}

	fn key_to_value(lua: &Lua, key: DataKey) -> mlua::Result<Value> {
		match key {
			DataKey::String(Cow::Owned(s)) => lua.create_external_string(s).map(Value::String),
			DataKey::Url(u) => u.into_lua(lua),
			DataKey::Path(p) => p.into_lua(lua),
			DataKey::Bytes(b) => lua.create_external_string(b).map(Value::String),
			_ => Self::key_to_value_ref(lua, &key),
		}
	}

	fn key_to_value_ref(lua: &Lua, key: &DataKey) -> mlua::Result<Value> {
		Ok(match key {
			DataKey::Nil => Value::Nil,
			DataKey::Boolean(b) => Value::Boolean(*b),
			DataKey::Integer(i) => Value::Integer(*i),
			DataKey::Number(n) => Value::Number(n.0),
			DataKey::String(s) => Value::String(lua.create_string(&**s)?),
			DataKey::Id(i) => i.into_lua(lua)?,
			DataKey::Url(u) => u.clone().into_lua(lua)?,
			DataKey::Path(p) => p.clone().into_lua(lua)?,
			DataKey::Bytes(b) => Value::String(lua.create_string(b)?),
		})
	}
}
