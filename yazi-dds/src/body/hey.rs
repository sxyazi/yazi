use std::collections::HashMap;

use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};

use super::{Body, BodyHi};
use crate::Peer;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHey {
	pub peers:   HashMap<u64, Peer>,
	pub version: String,
}

impl BodyHey {
	#[inline]
	pub fn owned(peers: HashMap<u64, Peer>) -> Body<'static> {
		Self { peers, version: BodyHi::version() }.into()
	}
}

impl From<BodyHey> for Body<'_> {
	fn from(value: BodyHey) -> Self { Self::Hey(value) }
}

impl IntoLua for BodyHey {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
		Err("BodyHey cannot be converted to Lua").into_lua_err()
	}
}
