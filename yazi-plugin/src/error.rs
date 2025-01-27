use std::{borrow::Cow, fmt::Display};

use mlua::{ExternalError, FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};

const EXPECTED: &str = "expected a Error";

pub enum Error {
	Io(std::io::Error),
	Serde(serde_json::Error),
	Custom(String),
}

impl Error {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		let new = lua.create_function(|_, msg: String| Ok(Error::Custom(msg)))?;

		lua.globals().raw_set("Error", lua.create_table_from([("custom", new)])?)
	}

	fn to_string(&self) -> Cow<str> {
		match self {
			Error::Io(e) => Cow::Owned(e.to_string()),
			Error::Serde(e) => Cow::Owned(e.to_string()),
			Error::Custom(s) => Cow::Borrowed(s),
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Io(e) => write!(f, "{e}"),
			Error::Serde(e) => write!(f, "{e}"),
			Error::Custom(s) => write!(f, "{s}"),
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
				Error::Io(e) => e.raw_os_error(),
				_ => None,
			})
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
			lua.create_string(me.to_string().as_ref())
		});
		methods.add_meta_function(MetaMethod::Concat, |lua, (lhs, rhs): (Value, Value)| {
			match (lhs, rhs) {
				(Value::String(l), Value::UserData(r)) => {
					let r = r.borrow::<Self>()?;
					lua.create_string([l.as_bytes().as_ref(), r.to_string().as_bytes()].concat())
				}
				(Value::UserData(l), Value::String(r)) => {
					let l = l.borrow::<Self>()?;
					lua.create_string([l.to_string().as_bytes(), r.as_bytes().as_ref()].concat())
				}
				_ => Err("only string can be concatenated with Error".into_lua_err()),
			}
		});
	}
}
