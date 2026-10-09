use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug)]
pub struct PasteOpt {
	pub(crate) before: bool,
}

impl<C> FromAction<C> for PasteOpt {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { before: a.bool("before") })
	}
}

impl FromLua for PasteOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PasteOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
