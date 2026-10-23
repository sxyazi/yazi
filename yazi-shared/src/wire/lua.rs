use std::collections::BTreeMap;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};

use crate::{id::Id, url::UrlBuf, wire::{Wire, WireKey}};

impl FromLua for Wire {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match &value {
			Value::Nil => Self::Nil,
			Value::Boolean(b) => (*b).into(),
			Value::Integer(i) => (*i).into(),
			Value::Number(n) => (*n).into(),
			Value::String(s) => match s.to_str() {
				Ok(s) => s.to_owned().into(),
				Err(_) => s.as_bytes().to_owned().into(),
			},
			Value::Table(t) => {
				let mut list = Vec::with_capacity(t.raw_len());
				let mut dict = BTreeMap::new();
				for pair in t.pairs::<WireKey, Self>() {
					let (k, v) = pair?;
					if dict.is_empty() && k == (list.len() as i64 + 1).into() {
						list.push(v);
					} else if !dict.is_empty() {
						dict.insert(k, v);
					} else {
						dict.extend(list.drain(..).enumerate().map(|(i, v)| ((i as i64 + 1).into(), v)));
						dict.insert(k, v);
					}
				}
				if dict.is_empty() { list.into() } else { dict.into() }
			}
			Value::UserData(ud) if let Ok(url) = ud.take::<UrlBuf>() => Self::Url(url),
			Value::UserData(ud) if let Ok(id) = ud.borrow::<Id>() => (*id).into(),
			_ => return Err(format!("unsupported value included: {value:?}").into_lua_err()),
		})
	}
}

impl IntoLua for Wire {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Ok(match self {
			Self::String(s) => Value::String(lua.create_external_string(s)?),
			Self::List(l) => Value::Table(lua.create_sequence_from(l)?),
			Self::Dict(d) => {
				let tbl = lua.create_table_with_capacity(0, d.len())?;
				for (k, v) in d {
					tbl.raw_set(k, v)?;
				}
				Value::Table(tbl)
			}
			Self::Url(u) => u.into_lua(lua)?,
			Self::Bytes(b) => Value::String(lua.create_external_string(b)?),
			data => (&data).into_lua(lua)?,
		})
	}
}

impl IntoLua for &Wire {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Ok(match self {
			Wire::Nil => Value::Nil,
			Wire::Boolean(b) => Value::Boolean(*b),
			Wire::Integer(i) => Value::Integer(*i),
			Wire::Number(n) => Value::Number(n.0),
			Wire::String(s) => Value::String(lua.create_string(s)?),
			Wire::List(l) => Value::Table(lua.create_sequence_from(l)?),
			Wire::Dict(d) => {
				let tbl = lua.create_table_with_capacity(0, d.len())?;
				for (k, v) in d {
					tbl.raw_set(k, v)?;
				}
				Value::Table(tbl)
			}
			Wire::Id(i) => i.into_lua(lua)?,
			Wire::Url(u) => u.clone().into_lua(lua)?,
			Wire::Bytes(b) => Value::String(lua.create_string(b)?),
		})
	}
}
