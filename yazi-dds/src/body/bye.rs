use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyBye {}

impl BodyBye {
	#[inline]
	pub fn borrowed() -> Body<'static> { Self {}.into() }
}

impl<'a> From<BodyBye> for Body<'a> {
	fn from(value: BodyBye) -> Self { Self::Bye(value) }
}

impl IntoLua<'_> for BodyBye {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value<'_>> {
		Err("BodyBye cannot be converted to Lua").into_lua_err()
	}
}
