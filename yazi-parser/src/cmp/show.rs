use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::cmp::CmpOpt;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug)]
pub struct ShowForm {
	pub opt: CmpOpt,
}

impl From<CmpOpt> for ShowForm {
	fn from(opt: CmpOpt) -> Self { Self { opt } }
}

impl TryFrom<ActionCow> for ShowForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: a.take_any("opt").ok_or_else(|| anyhow!("Invalid 'opt' in ShowForm"))? })
	}
}

impl FromLua for ShowForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShowForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
