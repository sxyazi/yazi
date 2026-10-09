use hashbrown::HashMap;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{event::{ActionCow, FromAction}, url::UrlCow};
use yazi_shim::SStr;

#[derive(Debug, Deserialize)]
pub struct UpdateMimesForm {
	pub updates: HashMap<UrlCow<'static>, SStr>,
}

impl<C> FromAction<C> for UpdateMimesForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for UpdateMimesForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateMimesForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
