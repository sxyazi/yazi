use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::ActionCow;

use crate::input::Gait;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ForwardOpt {
	#[serde(alias = "0", default)]
	pub(crate) gait:        Gait,
	#[serde(default)]
	pub(crate) end_of_word: bool,
}

impl TryFrom<ActionCow> for ForwardOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for ForwardOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ForwardOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
