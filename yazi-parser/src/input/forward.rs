use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::CmdCow;

#[derive(Debug)]
pub struct ForwardOpt {
	pub far:         bool,
	pub end_of_word: bool,
}

impl From<CmdCow> for ForwardOpt {
	fn from(c: CmdCow) -> Self { Self { far: c.bool("far"), end_of_word: c.bool("end-of-word") } }
}

impl FromLua for ForwardOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ForwardOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
