use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::ActionCow;

use crate::input::Gait;

#[derive(Debug, Deserialize)]
pub struct BackwardOpt {
	#[serde(alias = "0", default)]
	pub(crate) gait: Gait,
}

impl TryFrom<ActionCow> for BackwardOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for BackwardOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for BackwardOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
