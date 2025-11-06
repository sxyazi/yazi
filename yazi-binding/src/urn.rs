use std::ops::Deref;

use mlua::{ExternalError, FromLua, Lua, UserData, Value};
use yazi_shared::path::PathBufDyn;

pub struct Path(pub PathBufDyn);

impl Deref for Path {
	type Target = PathBufDyn;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Path {
	pub fn new(urn: impl Into<PathBufDyn>) -> Self { Self(urn.into()) }
}

impl FromLua for Path {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => ud.take()?,
			_ => Err("Expected a Path".into_lua_err())?,
		})
	}
}

impl UserData for Path {}
