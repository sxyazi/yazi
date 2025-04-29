use std::ops::Deref;

use mlua::{ExternalError, FromLua, Lua, UserData, Value};

pub struct Urn {
	inner: yazi_shared::url::UrnBuf,
}

impl Deref for Urn {
	type Target = yazi_shared::url::UrnBuf;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<Urn> for yazi_shared::url::UrnBuf {
	fn from(value: Urn) -> Self { value.inner }
}

impl Urn {
	pub fn new(urn: yazi_shared::url::UrnBuf) -> Self { Self { inner: urn } }
}

impl FromLua for Urn {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => ud.take()?,
			_ => Err("Expected a Urn".into_lua_err())?,
		})
	}
}

impl UserData for Urn {}
