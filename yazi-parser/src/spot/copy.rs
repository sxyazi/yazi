use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::ActionCow};

#[derive(Debug)]
pub struct CopyOpt {
	pub r#type: SStr,
}

impl From<ActionCow> for CopyOpt {
	fn from(mut a: ActionCow) -> Self { Self { r#type: a.take_first().unwrap_or_default() } }
}

impl FromLua for CopyOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CopyOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
