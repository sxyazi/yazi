use std::fmt::Debug;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::app::PluginOpt;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug, Default)]
pub struct PluginForm {
	pub opt: PluginOpt,
}

impl From<PluginOpt> for PluginForm {
	fn from(opt: PluginOpt) -> Self { Self { opt } }
}

impl TryFrom<ActionCow> for PluginForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self { opt: if let Some(opt) = a.take_any("opt") { opt } else { a.try_into()? } })
	}
}

impl FromLua for PluginForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PluginForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
