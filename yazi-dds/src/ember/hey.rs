use hashbrown::HashMap;
use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::{Id, SStr};

use super::{Ember, EmberHi};
use crate::Peer;

/// Server handshake
#[derive(Debug, Deserialize, Serialize)]
pub struct EmberHey {
	pub peers:   HashMap<Id, Peer>,
	pub version: SStr,
}

impl EmberHey {
	pub fn owned(peers: HashMap<Id, Peer>) -> Ember<'static> {
		Self { peers, version: EmberHi::version().into() }.into()
	}
}

impl From<EmberHey> for Ember<'_> {
	fn from(value: EmberHey) -> Self { Self::Hey(value) }
}

impl IntoLua for EmberHey {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
		Err("BodyHey cannot be converted to Lua").into_lua_err()
	}
}
