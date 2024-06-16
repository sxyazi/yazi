use std::{borrow::Cow, collections::HashSet};

use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;

/// The client handshake
#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHi<'a> {
	/// Specifies the kinds of events that the client can handle
	pub abilities: HashSet<Cow<'a, str>>,
	pub version:   String,
}

impl<'a> BodyHi<'a> {
	#[inline]
	pub fn borrowed(abilities: HashSet<&'a str>) -> Body<'a> {
		Self {
			abilities: abilities.into_iter().map(Cow::Borrowed).collect(),
			version:   Self::version(),
		}
		.into()
	}

	#[inline]
	pub fn version() -> String { format!("{} {}", env!("CARGO_PKG_VERSION"), env!("VERGEN_GIT_SHA")) }
}

impl<'a> From<BodyHi<'a>> for Body<'a> {
	fn from(value: BodyHi<'a>) -> Self { Self::Hi(value) }
}

impl IntoLua<'_> for BodyHi<'_> {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value<'_>> {
		Err("BodyHi cannot be converted to Lua").into_lua_err()
	}
}
