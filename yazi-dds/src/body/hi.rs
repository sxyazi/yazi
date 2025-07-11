use std::{borrow::Cow, collections::HashSet};

use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::SStr;

use super::Body;

/// Client handshake
#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHi<'a> {
	/// Kinds of events the client can handle
	pub abilities: HashSet<Cow<'a, str>>,
	pub version:   SStr,
}

impl<'a> BodyHi<'a> {
	pub fn borrowed<I>(abilities: I) -> Body<'a>
	where
		I: Iterator<Item = &'a str>,
	{
		Self { abilities: abilities.map(Into::into).collect(), version: Self::version().into() }.into()
	}

	pub fn version() -> &'static str {
		concat!(env!("CARGO_PKG_VERSION"), " ", env!("VERGEN_GIT_SHA"))
	}
}

impl<'a> From<BodyHi<'a>> for Body<'a> {
	fn from(value: BodyHi<'a>) -> Self { Self::Hi(value) }
}

impl IntoLua for BodyHi<'_> {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
		Err("BodyHi cannot be converted to Lua").into_lua_err()
	}
}
