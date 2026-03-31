use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::ActionCow};

#[derive(Debug)]
pub struct CopyForm {
	pub r#type: SStr,
}

impl From<ActionCow> for CopyForm {
	fn from(mut a: ActionCow) -> Self { Self { r#type: a.take_first().unwrap_or_default() } }
}

impl FromLua for CopyForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CopyForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
