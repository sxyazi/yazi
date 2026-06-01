use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::ActionCow;

use crate::input::Gait;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ForwardOpt {
	#[serde(alias = "0", default)]
	pub gait:        Gait,
	#[serde(default)]
	pub end_of_word: bool,
}

impl TryFrom<ActionCow> for ForwardOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		// TODO: remove
		if a.bool("far") {
			static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
			if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
				yazi_macro::emit!(Call(yazi_shared::event::Action::new_relay("app:deprecate").with(
					"content",
					"`forward --far` is deprecated, use `forward wide` under `[input]` instead".to_string()
				)));
			}

			return Ok(Self { gait: Gait::Wide, end_of_word: a.bool("end-of-word") });
		}
		Ok(a.deserialize()?)
	}
}

impl FromLua for ForwardOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ForwardOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
