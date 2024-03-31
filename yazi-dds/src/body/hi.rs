use std::{borrow::Cow, collections::HashSet};

use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHi<'a> {
	pub abilities: HashSet<Cow<'a, String>>,
}

impl<'a> BodyHi<'a> {
	#[inline]
	pub fn borrowed(abilities: HashSet<&'a String>) -> Body<'a> {
		Self { abilities: abilities.into_iter().map(Cow::Borrowed).collect() }.into()
	}
}

impl<'a> From<BodyHi<'a>> for Body<'a> {
	fn from(value: BodyHi<'a>) -> Self { Self::Hi(value) }
}

impl IntoLua<'_> for BodyHi<'_> {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value<'_>> {
		Err("BodyHi cannot be converted to Lua").into_lua_err()
	}
}
