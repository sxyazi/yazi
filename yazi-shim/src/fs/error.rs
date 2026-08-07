use std::{fmt, io, sync::Arc};

use anyhow::Result;
use mlua::{AnyUserData, ExternalError, Lua, LuaString, MetaMethod, Table, UserData, UserDataFields, UserDataMethods, Value};
use yazi_codegen::FromLuaOwned;

use crate::{fs::{kind_from_str, kind_to_str}, log::LOG_LEVEL};

#[derive(Clone, Debug, Eq, FromLuaOwned, PartialEq)]
pub enum Error {
	Kind(io::ErrorKind),
	Raw(i32),
	Custom { kind: io::ErrorKind, code: Option<i32>, message: Arc<str> },
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self {
		if err.get_ref().is_some() {
			Self::Custom {
				kind:    err.kind(),
				code:    err.raw_os_error(),
				message: err.to_string().into(),
			}
		} else if let Some(code) = err.raw_os_error() {
			Self::Raw(code)
		} else {
			Self::Kind(err.kind())
		}
	}
}

impl From<io::ErrorKind> for Error {
	fn from(kind: io::ErrorKind) -> Self { Self::Kind(kind) }
}

impl From<Error> for io::Error {
	fn from(value: Error) -> Self {
		match value {
			Error::Kind(kind) => Self::from(kind),
			Error::Raw(code) => Self::from_raw_os_error(code),
			Error::Custom { kind, message, .. } => Self::new(kind, message.to_string()),
		}
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Kind(kind) => io::Error::from(*kind).fmt(f),
			Self::Raw(code) => io::Error::from_raw_os_error(*code).fmt(f),
			Self::Custom { message, .. } => write!(f, "{message}"),
		}
	}
}

impl Error {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		let new =
			lua.create_function(|_, (_, ud): (Table, AnyUserData)| Ok(ud.borrow::<Self>()?.clone()))?;

		let fs = lua.create_function(|_, value: Value| {
			Ok(match value {
				Value::Table(t) => Self::custom(
					&t.raw_get::<LuaString>("kind")?.to_str()?,
					t.raw_get("code")?,
					&t.raw_get::<LuaString>("message")?.to_str()?,
				)?,
				_ => Err("expected a table".into_lua_err())?,
			})
		})?;
		let other = lua.create_function(|_, msg: String| Ok(Self::other(msg)))?;

		let error = lua.create_table_from([("fs", fs), ("other", other)])?;
		error.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		lua.globals().raw_set("Error", error)
	}

	pub fn other(message: impl Into<Arc<str>>) -> Self {
		Self::Custom { kind: io::ErrorKind::Other, code: None, message: message.into() }
	}

	pub fn custom(kind: &str, code: Option<i32>, message: &str) -> Result<Self> {
		Ok(Self::Custom { kind: kind_from_str(kind)?, code, message: message.into() })
	}

	pub fn kind(&self) -> io::ErrorKind {
		match self {
			Self::Kind(kind) => *kind,
			Self::Raw(code) => io::Error::from_raw_os_error(*code).kind(),
			Self::Custom { kind, .. } => *kind,
		}
	}

	pub fn kind_str(&self) -> &'static str { kind_to_str(self.kind()) }

	pub fn raw_os_error(&self) -> Option<i32> {
		match self {
			Self::Kind(_) => None,
			Self::Raw(code) => Some(*code),
			Self::Custom { code, .. } => *code,
		}
	}
}

impl UserData for Error {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("code", |_, me| Ok(me.raw_os_error()));
		fields.add_field_method_get("kind", |_, me| Ok(Some(me.kind_str())));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
			lua.create_external_string(me.to_string())
		});
		methods.add_meta_function(MetaMethod::Concat, |lua, (lhs, rhs): (Value, Value)| {
			match (lhs, rhs) {
				(Value::String(l), Value::UserData(r)) => {
					let r = r.borrow::<Self>()?;
					lua.create_external_string([&l.as_bytes(), r.to_string().as_bytes()].concat())
				}
				(Value::UserData(l), Value::String(r)) => {
					let l = l.borrow::<Self>()?;
					lua.create_external_string([l.to_string().as_bytes(), &r.as_bytes()].concat())
				}
				_ => Err("only string can be concatenated with Error".into_lua_err()),
			}
		});

		if !LOG_LEVEL.get().is_none() {
			methods.add_meta_function(MetaMethod::ToDebugString, |_, ud: AnyUserData| {
				Ok(format!("Error({:?}): {:?}", ud.to_pointer(), *ud.borrow::<Self>()?))
			});
		}
	}
}
