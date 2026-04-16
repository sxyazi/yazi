use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Debug, Deserialize)]
pub struct BulkExitForm {
	#[serde(alias = "0")]
	pub target: UrlBuf,
	#[serde(default)]
	pub accept: bool,
}

impl TryFrom<ActionCow> for BulkExitForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for BulkExitForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for BulkExitForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
