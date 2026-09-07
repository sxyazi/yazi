use std::{any::TypeId, borrow::Cow};

use mlua::{ExternalError, IntoLua, Lua, Value};
use ordered_float::OrderedFloat;

use super::Sendable;
use crate::{data::DataKey, id::Id, path::PathBufDyn, url::UrlBuf};

impl Sendable {
	pub(super) fn value_to_key(value: Value) -> mlua::Result<DataKey> {
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

	pub(super) fn key_to_value(lua: &Lua, key: DataKey) -> mlua::Result<Value> {
		match key {
			DataKey::String(Cow::Owned(s)) => lua.create_external_string(s).map(Value::String),
			DataKey::Url(u) => u.into_lua(lua),
			DataKey::Path(p) => p.into_lua(lua),
			DataKey::Bytes(b) => lua.create_external_string(b).map(Value::String),
			_ => Self::key_to_value_ref(lua, &key),
		}
	}

	pub(super) fn key_to_value_ref(lua: &Lua, key: &DataKey) -> mlua::Result<Value> {
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
