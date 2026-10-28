use anyhow::anyhow;
use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_core::app::PluginOpt;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug)]
pub struct PluginDoForm {
	pub opt: PluginOpt,
}

impl From<PluginOpt> for PluginDoForm {
	fn from(opt: PluginOpt) -> Self { Self { opt } }
}

impl<C> FromAction<C> for PluginDoForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { opt: a.take_any("opt").ok_or_else(|| anyhow!("Invalid 'opt' in PluginDoForm"))? })
	}
}

impl IntoLua for PluginDoForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
