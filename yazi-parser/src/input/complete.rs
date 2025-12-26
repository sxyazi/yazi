use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{Id, event::CmdCow};

use crate::cmp::CmpItem;

#[derive(Debug)]
pub struct CompleteOpt {
	pub item:   CmpItem,
	pub ticket: Id,
}

impl TryFrom<CmdCow> for CompleteOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(item) = c.take_any("item") else {
			bail!("Invalid 'item' in CompleteOpt");
		};

		Ok(Self { item, ticket: c.get("ticket").unwrap_or_default() })
	}
}

impl FromLua for CompleteOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CompleteOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
