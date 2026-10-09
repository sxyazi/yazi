use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_scheduler::process::ShellOpt;
use yazi_shared::event::{ActionCow, FromAction, Replier};

#[derive(Clone, Debug)]
pub struct ProcessOpenForm {
	pub opt:     ShellOpt,
	pub replier: Option<Replier>,
}

impl<C> FromAction<C> for ProcessOpenForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			opt:     a.take_any("opt").ok_or_else(|| anyhow!("Invalid 'opt' in ProcessOpenForm"))?,
			replier: a.take_replier(),
		})
	}
}

impl FromLua for ProcessOpenForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ProcessOpenForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
