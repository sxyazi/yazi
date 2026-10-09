use std::fmt::Debug;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::{Ctx, app::PluginOpt};
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug)]
pub struct PluginForm {
	pub opt: PluginOpt,
}

impl From<PluginOpt> for PluginForm {
	fn from(opt: PluginOpt) -> Self { Self { opt } }
}

impl FromAction<Ctx<'_>> for PluginForm {
	fn from_action(mut a: ActionCow, cx: &Ctx) -> anyhow::Result<Self> {
		if let Some(opt) = a.take_any("opt") {
			Ok(Self { opt })
		} else {
			Ok(Self { opt: PluginOpt::from_action(a, cx)? })
		}
	}
}

impl FromLua for PluginForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for PluginForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
