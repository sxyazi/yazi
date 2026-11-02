use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug, Default, Deserialize)]
pub struct BackspaceOpt {
	#[serde(default)]
	pub(crate) under: bool,
}

impl<C> FromAction<C> for BackspaceOpt {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl From<bool> for BackspaceOpt {
	fn from(under: bool) -> Self { Self { under } }
}

impl FromLua for BackspaceOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for BackspaceOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
