use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{Id, event::ActionCow, url::UrlBuf};

#[derive(Debug, Deserialize)]
pub struct UpdateSucceedForm {
	#[serde(alias = "0")]
	pub id:    Id,
	pub urls:  Vec<UrlBuf>,
	#[serde(default)]
	pub track: bool,
}

impl TryFrom<ActionCow> for UpdateSucceedForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for UpdateSucceedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateSucceedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
