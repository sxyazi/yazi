use std::borrow::Cow;

use hashbrown::HashSet;
use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::SStr;

use super::Ember;

/// Client handshake
#[derive(Debug, Deserialize, Serialize)]
pub struct EmberHi<'a> {
	/// Kinds of events the client can handle
	pub abilities: HashSet<Cow<'a, str>>,
	pub version:   SStr,
}

impl<'a> EmberHi<'a> {
	pub fn borrowed<I>(abilities: I) -> Ember<'a>
	where
		I: Iterator<Item = &'a str>,
	{
		Self { abilities: abilities.map(Into::into).collect(), version: Self::version().into() }.into()
	}

	pub fn version() -> &'static str {
		concat!(env!("CARGO_PKG_VERSION"), " ", env!("VERGEN_GIT_SHA"))
	}
}

impl<'a> From<EmberHi<'a>> for Ember<'a> {
	fn from(value: EmberHi<'a>) -> Self { Self::Hi(value) }
}

impl IntoLua for EmberHi<'_> {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
		Err("BodyHi cannot be converted to Lua").into_lua_err()
	}
}
