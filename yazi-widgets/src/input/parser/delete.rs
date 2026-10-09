use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug)]
pub struct DeleteOpt {
	pub(crate) cut:    bool,
	pub(crate) insert: bool,
}

impl<C> FromAction<C> for DeleteOpt {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { cut: a.bool("cut"), insert: a.bool("insert") })
	}
}

impl FromLua for DeleteOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DeleteOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
