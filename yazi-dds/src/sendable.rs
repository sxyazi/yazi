use std::collections::HashMap;

use mlua::{ExternalError, IntoLua, Lua, Value, Variadic};
use serde::{Deserialize, Serialize};
use yazi_shared::OrderedFloat;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueSendable {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(f64),
	String(String),
	Table(HashMap<ValueSendableKey, ValueSendable>),
}

impl ValueSendable {
	pub fn try_from_variadic(values: Variadic<Value>) -> mlua::Result<Vec<Self>> {
		let mut vec = Vec::with_capacity(values.len());
		for value in values {
			vec.push(Self::try_from(value)?);
		}
		Ok(vec)
	}

	pub fn into_table_string(self) -> HashMap<String, String> {
		let Self::Table(table) = self else {
			return Default::default();
		};

		let mut map = HashMap::with_capacity(table.len());
		for pair in table {
			if let (ValueSendableKey::String(k), Self::String(v)) = pair {
				map.insert(k, v);
			}
		}
		map
	}
}

impl<'a> TryFrom<Value<'a>> for ValueSendable {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		Ok(match value {
			Value::Nil => Self::Nil,
			Value::Boolean(b) => Self::Boolean(b),
			Value::LightUserData(_) => Err("light userdata is not supported".into_lua_err())?,
			Value::Integer(n) => Self::Integer(n),
			Value::Number(n) => Self::Number(n),
			Value::String(s) => Self::String(s.to_str()?.to_owned()),
			Value::Table(t) => {
				let mut map = HashMap::with_capacity(t.len().map(|l| l as usize)?);
				for result in t.pairs::<Value, Value>() {
					let (k, v) = result?;
					map.insert(Self::try_from(k)?.try_into()?, v.try_into()?);
				}
				Self::Table(map)
			}
			Value::Function(_) => Err("function is not supported".into_lua_err())?,
			Value::Thread(_) => Err("thread is not supported".into_lua_err())?,
			Value::UserData(_) => Err("userdata is not supported".into_lua_err())?,
			Value::Error(_) => Err("error is not supported".into_lua_err())?,
		})
	}
}

impl IntoLua<'_> for ValueSendable {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Self::Nil => Ok(Value::Nil),
			Self::Boolean(v) => Ok(Value::Boolean(v)),
			Self::Integer(v) => Ok(Value::Integer(v)),
			Self::Number(v) => Ok(Value::Number(v)),
			Self::String(v) => Ok(Value::String(lua.create_string(v)?)),
			Self::Table(v) => {
				let seq_len = v.keys().filter(|&k| !k.is_numeric()).count();
				let table = lua.create_table_with_capacity(seq_len, v.len() - seq_len)?;
				for (k, v) in v {
					table.raw_set(k, v)?;
				}
				Ok(Value::Table(table))
			}
		}
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueSendableKey {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(OrderedFloat),
	String(String),
}

impl ValueSendableKey {
	#[inline]
	fn is_numeric(&self) -> bool { matches!(self, Self::Integer(_) | Self::Number(_)) }
}

impl TryInto<ValueSendableKey> for ValueSendable {
	type Error = mlua::Error;

	fn try_into(self) -> Result<ValueSendableKey, Self::Error> {
		Ok(match self {
			Self::Nil => ValueSendableKey::Nil,
			Self::Boolean(v) => ValueSendableKey::Boolean(v),
			Self::Integer(v) => ValueSendableKey::Integer(v),
			Self::Number(v) => ValueSendableKey::Number(OrderedFloat::new(v)),
			Self::String(v) => ValueSendableKey::String(v),
			Self::Table(_) => Err("table is not supported".into_lua_err())?,
		})
	}
}

impl IntoLua<'_> for ValueSendableKey {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Self::Nil => Ok(Value::Nil),
			Self::Boolean(k) => Ok(Value::Boolean(k)),
			Self::Integer(k) => Ok(Value::Integer(k)),
			Self::Number(k) => Ok(Value::Number(k.get())),
			Self::String(k) => Ok(Value::String(lua.create_string(k)?)),
		}
	}
}
