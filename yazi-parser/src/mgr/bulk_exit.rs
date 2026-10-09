use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{event::{ActionCow, FromAction}, url::UrlBuf};

#[derive(Debug, Deserialize)]
pub struct BulkExitForm {
	#[serde(alias = "0")]
	pub target: UrlBuf,
	#[serde(default)]
	pub accept: bool,
}

impl<C> FromAction<C> for BulkExitForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for BulkExitForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for BulkExitForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
