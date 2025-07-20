use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::event::{CmdCow, Data};

#[derive(Debug)]
pub struct TabSwitchOpt {
	pub step:     isize,
	pub relative: bool,
}

impl From<CmdCow> for TabSwitchOpt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(Data::as_isize).unwrap_or(0), relative: c.bool("relative") }
	}
}

impl IntoLua for &TabSwitchOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
