use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::event::{CmdCow, EventQuit};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct QuitOpt {
	pub code:        i32,
	pub no_cwd_file: bool,
}

impl From<CmdCow> for QuitOpt {
	fn from(c: CmdCow) -> Self {
		Self { code: c.get("code").unwrap_or_default(), no_cwd_file: c.bool("no-cwd-file") }
	}
}

impl From<QuitOpt> for EventQuit {
	fn from(value: QuitOpt) -> Self {
		Self { code: value.code, no_cwd_file: value.no_cwd_file, ..Default::default() }
	}
}

impl FromLua for QuitOpt {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { lua.from_value(value) }
}

impl IntoLua for QuitOpt {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value(&self) }
}
