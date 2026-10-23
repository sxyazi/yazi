use std::{any::TypeId, borrow::Cow};

use hashbrown::HashMap;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use ordered_float::OrderedFloat;

use crate::{any_data::AnyData, data::{Data, DataInventory, DataKey}, id::Id, path::PathBufDyn, url::UrlBuf};

impl FromLua for Data {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		match &value {
			Value::Nil => return Ok(Self::Nil),
			Value::Boolean(b) => return Ok(Self::Boolean(*b)),
			Value::Integer(i) => return Ok(Self::Integer(*i)),
			Value::Number(n) => return Ok(Self::Number(*n)),
			Value::String(b) => {
				return Ok(if let Ok(s) = b.to_str() {
					Self::String(s.to_owned().into())
				} else {
					Self::Bytes(b.as_bytes().to_owned())
				});
			}
			Value::Table(t) => {
				let (mut i, mut map) = (1, HashMap::with_capacity(t.raw_len()));
				for result in t.pairs::<DataKey, Value>() {
					let (k, v) = result?;
					if k == DataKey::Integer(i) {
						i += 1;
					}
					map.insert(k, Self::from_lua(v, lua)?);
				}

				return Ok(if map.len() == i as usize - 1 {
					Self::List((1..i).map(|i| map.remove(&DataKey::Integer(i)).unwrap()).collect())
				} else {
					Self::Dict(map)
				});
			}
			Value::UserData(ud) => match ud.type_id() {
				Some(t) if t == TypeId::of::<UrlBuf>() => {
					return Ok(Self::Url(ud.take()?));
				}
				Some(t) if t == TypeId::of::<PathBufDyn>() => {
					return Ok(Self::Path(ud.take()?));
				}
				Some(t) if t == TypeId::of::<Id>() => {
					return Ok(Self::Id(*ud.borrow::<Id>()?));
				}
				Some(t) if t == TypeId::of::<AnyData>() => return Ok(Self::Any(ud.take::<AnyData>()?.0)),
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
				Ok(data) => return Ok(Self::Any(data)),
				Err(mlua::Error::UserDataTypeMismatch) => continue,
				Err(e) => return Err(e),
			}
		}
		Err(format!("unsupported value included: {value:?}").into_lua_err())?
	}
}

impl IntoLua for Data {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Ok(match self {
			Self::String(Cow::Owned(s)) => Value::String(lua.create_external_string(s)?),
			Self::List(l) => Value::Table(lua.create_sequence_from(l)?),
			Self::Dict(d) => {
				let seq_len = d.keys().filter(|&k| k.is_integer()).count();
				let tbl = lua.create_table_with_capacity(seq_len, d.len() - seq_len)?;
				for (k, v) in d {
					tbl.raw_set(k, v)?;
				}
				Value::Table(tbl)
			}
			Self::Url(u) => u.into_lua(lua)?,
			Self::Path(p) => p.into_lua(lua)?,
			Self::Bytes(b) => Value::String(lua.create_external_string(b)?),
			Self::Any(a) => a.into_lua(lua)?,
			data => (&data).into_lua(lua)?,
		})
	}
}

impl IntoLua for &Data {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Ok(match self {
			Data::Nil => Value::Nil,
			Data::Boolean(b) => Value::Boolean(*b),
			Data::Integer(i) => Value::Integer(*i),
			Data::Number(n) => Value::Number(*n),
			Data::String(s) => Value::String(lua.create_string(&**s)?),
			Data::List(l) => Value::Table(lua.create_sequence_from(l)?),
			Data::Dict(d) => {
				let seq_len = d.keys().filter(|&k| k.is_integer()).count();
				let tbl = lua.create_table_with_capacity(seq_len, d.len() - seq_len)?;
				for (k, v) in d {
					tbl.raw_set(k, v)?;
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
}

// --- DataKey
impl FromLua for DataKey {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
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
}

impl IntoLua for DataKey {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			DataKey::String(Cow::Owned(s)) => lua.create_external_string(s).map(Value::String),
			DataKey::Url(u) => u.into_lua(lua),
			DataKey::Path(p) => p.into_lua(lua),
			DataKey::Bytes(b) => lua.create_external_string(b).map(Value::String),
			key => (&key).into_lua(lua),
		}
	}
}

impl IntoLua for &DataKey {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Ok(match self {
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
