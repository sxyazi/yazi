use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug, Default)]
pub struct CloseForm {
	pub submit: bool,
}

impl<C> FromAction<C> for CloseForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { submit: a.bool("submit") })
	}
}

impl From<bool> for CloseForm {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl FromLua for CloseForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CloseForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
