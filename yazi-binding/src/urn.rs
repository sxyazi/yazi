use std::{ops::Deref, path::PathBuf};

use mlua::{ExternalError, FromLua, Lua, UserData, Value};
use yazi_shared::path::PathBufLike;

pub struct Urn<P: PathBufLike = PathBuf> {
	inner: yazi_shared::url::UrnBuf<P>,
}

impl<P> Deref for Urn<P>
where
	P: PathBufLike,
{
	type Target = yazi_shared::url::UrnBuf<P>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl<P> From<Urn<P>> for yazi_shared::url::UrnBuf<P>
where
	P: PathBufLike,
{
	fn from(value: Urn<P>) -> Self { value.inner }
}

impl<P> Urn<P>
where
	P: PathBufLike,
{
	pub fn new(urn: impl Into<yazi_shared::url::UrnBuf<P>>) -> Self { Self { inner: urn.into() } }
}

impl<P> FromLua for Urn<P>
where
	P: PathBufLike,
{
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => ud.take()?,
			_ => Err("Expected a Urn".into_lua_err())?,
		})
	}
}

impl<P> UserData for Urn<P> where P: PathBufLike {}
