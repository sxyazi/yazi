use std::collections::BTreeMap;

use mlua::{ExternalError, IntoLua, Lua, Value};

use super::Sendable;
use crate::{id::Id, url::UrlBuf, wire::{Wire, WireKey}};

impl Sendable {
	pub fn value_to_wire(lua: &Lua, value: Value) -> mlua::Result<Wire> {
		Ok(match &value {
			Value::Nil => Wire::Nil,
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
				for pair in t.pairs::<WireKey, Value>() {
					let (k, v) = pair?;
					let v = Self::value_to_wire(lua, v)?;
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
			Value::UserData(ud) if let Ok(url) = ud.take::<UrlBuf>() => Wire::Url(url),
			Value::UserData(ud) if let Ok(id) = ud.borrow::<Id>() => (*id).into(),
			_ => return Err(format!("unsupported value included: {value:?}").into_lua_err()),
		})
	}

	pub fn wire_to_value(lua: &Lua, data: Wire) -> mlua::Result<Value> {
		Ok(match data {
			Wire::String(s) => Value::String(lua.create_external_string(s)?),
			Wire::List(l) => {
				let mut vec = Vec::with_capacity(l.len());
				for v in l {
					vec.push(Self::wire_to_value(lua, v)?);
				}
				Value::Table(lua.create_sequence_from(vec)?)
			}
			Wire::Dict(d) => {
				let tbl = lua.create_table_with_capacity(0, d.len())?;
				for (k, v) in d {
					tbl.raw_set(k.into_lua(lua)?, Self::wire_to_value(lua, v)?)?;
				}
				Value::Table(tbl)
			}
			Wire::Url(u) => u.into_lua(lua)?,
			Wire::Bytes(b) => Value::String(lua.create_external_string(b)?),
			_ => Self::wire_to_value_ref(lua, &data)?,
		})
	}

	pub fn wire_to_value_ref(lua: &Lua, data: &Wire) -> mlua::Result<Value> {
		Ok(match data {
			Wire::Nil => Value::Nil,
			Wire::Boolean(b) => Value::Boolean(*b),
			Wire::Integer(i) => Value::Integer(*i),
			Wire::Number(n) => Value::Number(n.0),
			Wire::String(s) => Value::String(lua.create_string(s)?),
			Wire::List(l) => {
				let mut vec = Vec::with_capacity(l.len());
				for v in l {
					vec.push(Self::wire_to_value_ref(lua, v)?);
				}
				Value::Table(lua.create_sequence_from(vec)?)
			}
			Wire::Dict(d) => {
				let tbl = lua.create_table_with_capacity(0, d.len())?;
				for (k, v) in d {
					tbl.raw_set(k.into_lua(lua)?, Self::wire_to_value_ref(lua, v)?)?;
				}
				Value::Table(tbl)
			}
			Wire::Id(i) => i.into_lua(lua)?,
			Wire::Url(u) => u.clone().into_lua(lua)?,
			Wire::Bytes(b) => Value::String(lua.create_string(b)?),
		})
	}
}
