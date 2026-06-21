use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::ActionCow;

use crate::input::Gait;

#[derive(Debug, Deserialize)]
pub struct BackwardOpt {
	#[serde(alias = "0", default)]
	pub gait: Gait,
}

impl TryFrom<ActionCow> for BackwardOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		// TODO: remove
		if a.bool("far") {
			static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
			if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
				yazi_macro::emit!(Call(yazi_shared::event::Action::new_relay("app:deprecate").with(
					"content",
					"`backward --far` is deprecated, use `backward wide` under `[input]` instead".to_string()
				)));
			}

			return Ok(Self { gait: Gait::Wide });
		}
		Ok(a.deserialize()?)
	}
}

impl FromLua for BackwardOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for BackwardOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
