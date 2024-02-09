use std::collections::HashMap;

use mlua::{AnyUserData, ExternalError, IntoLua, Lua, Value, Variadic};
use yazi_shared::OrderedFloat;

use crate::elements::Renderable;

pub fn cast_to_renderable(ud: AnyUserData) -> Option<Box<dyn Renderable + Send>> {
	if let Ok(c) = ud.take::<crate::elements::Paragraph>() {
		Some(Box::new(c))
	} else if let Ok(c) = ud.take::<crate::elements::List>() {
		Some(Box::new(c))
	} else if let Ok(c) = ud.take::<crate::elements::Bar>() {
		Some(Box::new(c))
	} else if let Ok(c) = ud.take::<crate::elements::Border>() {
		Some(Box::new(c))
	} else if let Ok(c) = ud.take::<crate::elements::Gauge>() {
		Some(Box::new(c))
	} else {
		None
	}
}

#[derive(Debug)]
pub enum ValueSendable {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(f64),
	String(Vec<u8>),
	Table(HashMap<ValueSendableKey, ValueSendable>),
}

impl<'a> TryFrom<Value<'a>> for ValueSendable {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		Ok(match value {
			Value::Nil => ValueSendable::Nil,
			Value::Boolean(b) => ValueSendable::Boolean(b),
			Value::LightUserData(_) => Err("light userdata is not supported".into_lua_err())?,
			Value::Integer(n) => ValueSendable::Integer(n),
			Value::Number(n) => ValueSendable::Number(n),
			Value::String(s) => ValueSendable::String(s.as_bytes().to_vec()),
			Value::Table(t) => {
				let mut map = HashMap::with_capacity(t.len().map(|l| l as usize)?);
				for result in t.pairs::<Value, Value>() {
					let (k, v) = result?;
					map.insert(ValueSendable::try_from(k)?.try_into()?, v.try_into()?);
				}
				ValueSendable::Table(map)
			}
			Value::Function(_) => Err("function is not supported".into_lua_err())?,
			Value::Thread(_) => Err("thread is not supported".into_lua_err())?,
			Value::UserData(_) => Err("userdata is not supported".into_lua_err())?,
			Value::Error(_) => Err("error is not supported".into_lua_err())?,
		})
	}
}

impl<'lua> IntoLua<'lua> for ValueSendable {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			ValueSendable::Nil => Ok(Value::Nil),
			ValueSendable::Boolean(b) => Ok(Value::Boolean(b)),
			ValueSendable::Integer(n) => Ok(Value::Integer(n)),
			ValueSendable::Number(n) => Ok(Value::Number(n)),
			ValueSendable::String(s) => Ok(Value::String(lua.create_string(s)?)),
			ValueSendable::Table(t) => {
				let table = lua.create_table()?;
				for (k, v) in t {
					table.set(k.into_lua(lua)?, v.into_lua(lua)?)?;
				}
				Ok(Value::Table(table))
			}
		}
	}
}

impl ValueSendable {
	pub fn try_from_variadic(values: Variadic<Value>) -> mlua::Result<Vec<ValueSendable>> {
		let mut vec = Vec::with_capacity(values.len());
		for value in values {
			vec.push(ValueSendable::try_from(value)?);
		}
		Ok(vec)
	}

	pub fn into_table_string(self) -> HashMap<String, String> {
		let ValueSendable::Table(table) = self else {
			return Default::default();
		};

		let mut map = HashMap::with_capacity(table.len());
		for pair in table {
			let (ValueSendableKey::String(k), ValueSendable::String(v)) = pair else {
				continue;
			};
			if let (Ok(k), Ok(v)) = (String::from_utf8(k), String::from_utf8(v)) {
				map.insert(k, v);
			}
		}
		map
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ValueSendableKey {
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(OrderedFloat),
	String(Vec<u8>),
}

impl TryInto<ValueSendableKey> for ValueSendable {
	type Error = mlua::Error;

	fn try_into(self) -> Result<ValueSendableKey, Self::Error> {
		Ok(match self {
			ValueSendable::Nil => ValueSendableKey::Nil,
			ValueSendable::Boolean(b) => ValueSendableKey::Boolean(b),
			ValueSendable::Integer(n) => ValueSendableKey::Integer(n),
			ValueSendable::Number(n) => ValueSendableKey::Number(OrderedFloat::new(n)),
			ValueSendable::String(s) => ValueSendableKey::String(s),
			ValueSendable::Table(_) => Err("table is not supported".into_lua_err())?,
		})
	}
}

impl<'lua> IntoLua<'lua> for ValueSendableKey {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			ValueSendableKey::Nil => Ok(Value::Nil),
			ValueSendableKey::Boolean(b) => Ok(Value::Boolean(b)),
			ValueSendableKey::Integer(n) => Ok(Value::Integer(n)),
			ValueSendableKey::Number(n) => Ok(Value::Number(n.get())),
			ValueSendableKey::String(s) => Ok(Value::String(lua.create_string(s)?)),
		}
	}
}
