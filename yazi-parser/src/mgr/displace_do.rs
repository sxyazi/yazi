use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::mgr::DisplaceOpt;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug)]
pub struct DisplaceDoForm {
	pub opt: DisplaceOpt,
}

impl TryFrom<ActionCow> for DisplaceDoForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: a.take_any("opt").ok_or_else(|| anyhow!("Invalid 'opt' in DisplaceDoForm"))? })
	}
}

impl FromLua for DisplaceDoForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DisplaceDoForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
