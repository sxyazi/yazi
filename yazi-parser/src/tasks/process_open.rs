use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_scheduler::process::ShellOpt;
use yazi_shared::event::{ActionCow, Replier};

#[derive(Clone, Debug)]
pub struct ProcessOpenForm {
	pub opt:     ShellOpt,
	pub replier: Option<Replier>,
}

impl TryFrom<ActionCow> for ProcessOpenForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
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
