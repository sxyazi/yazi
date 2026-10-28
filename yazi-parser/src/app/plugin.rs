use std::fmt::Debug;

use anyhow::bail;
use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_core::{Ctx, app::PluginOpt};
use yazi_shared::{event::{Action, ActionCow, Cmd, FromAction}, scope::Scope};
use yazi_shim::SStr;

#[derive(Clone, Debug)]
pub struct PluginForm {
	pub opt:   PluginOpt,
	pub scope: Scope,
}

impl FromAction<Ctx<'_>> for PluginForm {
	fn from_action(mut a: ActionCow, cx: &Ctx) -> anyhow::Result<Self> {
		let name: SStr = a.take_first().unwrap_or_default();
		if name.is_empty() {
			bail!("plugin name cannot be empty");
		}

		let args = if let Ok(s) = a.second() {
			let (words, last) = yazi_shared::shell::unix::split(s, true)?;
			Cmd::parse_args(words, last)
		} else {
			a.take_second().unwrap_or_default()
		};

		Ok(Self {
			scope: a.scope.clone(),
			opt:   PluginOpt {
				args,
				mode: a.str("mode").parse().unwrap_or_default(),
				method: a.str("method").parse().unwrap_or_default(),
				scope: cx.scope.clone(),
				..PluginOpt::new(name)
			},
		})
	}
}

impl FromAction<Ctx<'_>, Action> for PluginForm {
	fn from_action(action: Action, cx: &Ctx) -> anyhow::Result<Self> {
		Self::from_action(ActionCow::Owned(action), cx)
	}
}

impl IntoLua for PluginForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
