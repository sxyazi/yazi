use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::ActionCow;

#[derive(Debug, Default)]
pub struct BulkRenameOpt {
	pub no_prompt: bool,
}

impl From<ActionCow> for BulkRenameOpt {
	fn from(a: ActionCow) -> Self {
		Self { no_prompt: a.bool("no-prompt") }
	}
}

impl FromLua for BulkRenameOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> {
		Err("unsupported".into_lua_err())
	}
}

impl IntoLua for BulkRenameOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
		Err("unsupported".into_lua_err())
	}
}
