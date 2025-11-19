use std::ops::Deref;

use mlua::{ExternalError, FromLua, Lua, MetaMethod, UserData, UserDataMethods, Value};
use yazi_shared::path::{PathBufDyn, PathLike};

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

impl UserData for Path {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods
			.add_meta_method(MetaMethod::ToString, |lua, me, ()| lua.create_string(me.encoded_bytes()));
	}
}
