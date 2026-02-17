use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct ForwardOpt {
	pub far:         bool,
	pub end_of_word: bool,
}

impl From<ActionCow> for ForwardOpt {
	fn from(a: ActionCow) -> Self { Self { far: a.bool("far"), end_of_word: a.bool("end-of-word") } }
}

impl FromLua for ForwardOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ForwardOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
