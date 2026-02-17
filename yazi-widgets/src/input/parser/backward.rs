use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct BackwardOpt {
	pub far: bool,
}

impl From<ActionCow> for BackwardOpt {
	fn from(a: ActionCow) -> Self { Self { far: a.bool("far") } }
}

impl FromLua for BackwardOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for BackwardOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
