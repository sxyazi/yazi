use std::{borrow::Cow, collections::HashMap};

use mlua::{ExternalResult, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::Id;

use super::{Body, BodyHi};
use crate::Peer;

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyHey {
	pub peers:   HashMap<Id, Peer>,
	pub version: Cow<'static, str>,
}

impl BodyHey {
	#[inline]
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
