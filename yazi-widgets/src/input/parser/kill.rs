use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::event::{ActionCow, FromAction};
use yazi_shim::SStr;

#[derive(Debug)]
pub struct KillOpt {
	pub(crate) kind: SStr,
}

impl<C> FromAction<C> for KillOpt {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { kind: a.take_first().unwrap_or_default() })
	}
}

impl FromLua for KillOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for KillOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
