use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{Id, SStr, event::CmdCow};

#[derive(Debug)]
pub struct TriggerOpt {
	pub word:   SStr,
	pub ticket: Option<Id>,
}

impl From<CmdCow> for TriggerOpt {
	fn from(mut c: CmdCow) -> Self {
		Self { word: c.take_first().unwrap_or_default(), ticket: c.get("ticket").ok() }
	}
}

impl FromLua for TriggerOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TriggerOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
