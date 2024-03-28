use std::collections::HashMap;

use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::Body;
use crate::Peer;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHey {
	pub peers: HashMap<u64, Peer>,
}

impl From<BodyHey> for Body<'_> {
	fn from(value: BodyHey) -> Self { Self::Hey(value) }
}

impl IntoLua<'_> for BodyHey {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value<'_>> {
		Err("BodyHey cannot be converted to Lua").into_lua_err()
	}
}
