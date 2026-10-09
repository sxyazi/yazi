use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::tasks::TaskOpt;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug)]
pub struct SpawnForm {
	pub opt: TaskOpt,
}

impl<C> FromAction<C> for SpawnForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { opt: a.take_any("opt").ok_or_else(|| anyhow!("Invalid 'opt' in SpawnForm"))? })
	}
}

impl FromLua for SpawnForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SpawnForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
