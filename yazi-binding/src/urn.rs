use std::{ops::Deref, path::PathBuf};

use mlua::{ExternalError, FromLua, Lua, UserData, Value};
use yazi_shared::path::PathBufLike;

pub struct Path<P: PathBufLike = PathBuf>(pub P);

impl<P> Deref for Path<P>
where
	P: PathBufLike,
{
	type Target = P;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P> Path<P>
where
	P: PathBufLike,
{
	pub fn new(urn: impl Into<P>) -> Self { Self(urn.into()) }
}

impl<P> FromLua for Path<P>
where
	P: PathBufLike,
{
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => ud.take()?,
			_ => Err("Expected a Path".into_lua_err())?,
		})
	}
}

impl<P> UserData for Path<P> where P: PathBufLike {}
