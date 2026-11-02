use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{event::{ActionCow, FromAction}, id::Id};
use yazi_shim::SStr;

#[derive(Debug, Deserialize)]
pub struct TriggerForm {
	#[serde(alias = "0", default)]
	pub word:   SStr,
	pub ticket: Option<Id>,
}

impl From<String> for TriggerForm {
	fn from(word: String) -> Self { Self { word: word.into(), ticket: None } }
}

impl<C> FromAction<C> for TriggerForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for TriggerForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TriggerForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
