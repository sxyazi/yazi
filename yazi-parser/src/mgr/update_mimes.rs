use hashbrown::HashMap;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{SStr, event::ActionCow, url::UrlCow};

#[derive(Debug, Deserialize)]
pub struct UpdateMimesForm {
	pub updates: HashMap<UrlCow<'static>, SStr>,
}

impl TryFrom<ActionCow> for UpdateMimesForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl FromLua for UpdateMimesForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateMimesForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
