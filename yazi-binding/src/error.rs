use std::{borrow::Cow, fmt::Display};

use mlua::{ExternalError, FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use yazi_shared::SStr;

const EXPECTED: &str = "expected a Error";

pub enum Error {
	Io(std::io::Error),
	Fs(yazi_fs::error::Error),
	Serde(serde_json::Error),
	Custom(SStr),
}

impl Error {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		let custom = lua.create_function(|_, msg: String| Ok(Self::custom(msg)))?;

		let fs = lua.create_function(|_, value: Value| {
			Ok(Self::Fs(match value {
				Value::Table(t) => yazi_fs::error::Error::custom(
					&t.raw_get::<mlua::String>("kind")?.to_str()?,
					t.raw_get("code")?,
					&t.raw_get::<mlua::String>("message")?.to_str()?,
				)?,
				_ => Err("expected a table".into_lua_err())?,
			}))
		})?;

		lua.globals().raw_set("Error", lua.create_table_from([("custom", custom), ("fs", fs)])?)
	}

	pub fn custom(msg: impl Into<SStr>) -> Self { Self::Custom(msg.into()) }

	pub fn into_string(self) -> SStr {
		match self {
			Self::Io(e) => Cow::Owned(e.to_string()),
			Self::Fs(e) => Cow::Owned(e.to_string()),
			Self::Serde(e) => Cow::Owned(e.to_string()),
			Self::Custom(s) => s,
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Io(e) => write!(f, "{e}"),
			Self::Fs(e) => write!(f, "{e}"),
			Self::Serde(e) => write!(f, "{e}"),
			Self::Custom(s) => write!(f, "{s}"),
		}
	}
}

impl FromLua for Error {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => ud.take(),
			_ => Err(EXPECTED.into_lua_err()),
		}
	}
}

impl UserData for Error {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("code", |_, me| {
			Ok(match me {
				Self::Io(e) => e.raw_os_error(),
				Self::Fs(e) => e.raw_os_error(),
				_ => None,
			})
		});
		fields.add_field_method_get("kind", |_, me| {
			Ok(match me {
				Self::Io(e) => Some(yazi_fs::error::Error::from(e.kind()).kind_str()),
				Self::Fs(e) => Some(e.kind_str()),
				_ => None,
			})
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
			Ok(match me {
				Self::Io(_) | Self::Fs(_) | Self::Serde(_) => lua.create_string(me.to_string()),
				Self::Custom(s) => lua.create_string(&**s),
			})
		});
		methods.add_meta_function(MetaMethod::Concat, |lua, (lhs, rhs): (Value, Value)| {
			match (lhs, rhs) {
				(Value::String(l), Value::UserData(r)) => {
					let r = r.borrow::<Self>()?;
					lua.create_string([&l.as_bytes(), r.to_string().as_bytes()].concat())
				}
				(Value::UserData(l), Value::String(r)) => {
					let l = l.borrow::<Self>()?;
					lua.create_string([l.to_string().as_bytes(), &r.as_bytes()].concat())
				}
				_ => Err("only string can be concatenated with Error".into_lua_err()),
			}
		});
	}
}
