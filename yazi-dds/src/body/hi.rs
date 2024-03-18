use std::collections::HashSet;

use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHi {
	pub id:        u64,
	pub abilities: HashSet<String>,
}

impl From<BodyHi> for Body<'_> {
	fn from(value: BodyHi) -> Self { Self::Hi(value) }
}

impl IntoLua<'_> for BodyHi {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value<'_>> {
		Err("BodyHi cannot be converted to Lua").into_lua_err()
	}
}
