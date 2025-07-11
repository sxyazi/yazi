use std::collections::HashMap;

use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{Id, SStr};

use super::{Body, BodyHi};
use crate::Peer;

/// Server handshake
#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHey {
	pub peers:   HashMap<Id, Peer>,
	pub version: SStr,
}

impl BodyHey {
	pub fn owned(peers: HashMap<Id, Peer>) -> Body<'static> {
		Self { peers, version: BodyHi::version().into() }.into()
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
