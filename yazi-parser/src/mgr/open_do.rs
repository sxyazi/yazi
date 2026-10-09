use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::mgr::OpenDoOpt;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug, Default)]
pub struct OpenDoForm {
	pub opt: OpenDoOpt,
}

impl From<OpenDoOpt> for OpenDoForm {
	fn from(opt: OpenDoOpt) -> Self { Self { opt } }
}

impl<C> FromAction<C> for OpenDoForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { opt: a.take_any("opt").ok_or_else(|| anyhow!("Invalid 'opt' in OpenDoForm"))? })
	}
}

impl FromLua for OpenDoForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for OpenDoForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
