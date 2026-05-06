use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct HistoryOpt {
	pub up: bool,
}

impl From<ActionCow> for HistoryOpt {
	fn from(a: ActionCow) -> Self { Self { up: a.str(0) == "up" } }
}

impl FromLua for HistoryOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for HistoryOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
