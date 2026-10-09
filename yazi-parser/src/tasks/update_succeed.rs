use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{event::{ActionCow, FromAction}, id::Id, url::UrlBuf};

#[derive(Debug, Deserialize)]
pub struct UpdateSucceedForm {
	#[serde(alias = "0")]
	pub id:    Id,
	pub urls:  Vec<UrlBuf>,
	#[serde(default)]
	pub track: bool,
}

impl<C> FromAction<C> for UpdateSucceedForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for UpdateSucceedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateSucceedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
