use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyBye;

impl BodyBye {
	#[inline]
	pub fn owned() -> Body<'static> { Self.into() }
}

impl<'a> From<BodyBye> for Body<'a> {
	fn from(value: BodyBye) -> Self { Self::Bye(value) }
}

impl IntoLua for BodyBye {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
		Err("BodyBye cannot be converted to Lua").into_lua_err()
	}
}
