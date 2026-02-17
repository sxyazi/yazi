use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug)]
pub struct InsertOpt {
	pub append: bool,
}

impl From<ActionCow> for InsertOpt {
	fn from(a: ActionCow) -> Self { Self { append: a.bool("append") } }
}

impl From<bool> for InsertOpt {
	fn from(append: bool) -> Self { Self { append } }
}

impl FromLua for InsertOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for InsertOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
