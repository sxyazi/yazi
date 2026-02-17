use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Debug, Default)]
pub struct UploadOpt {
	pub urls: Vec<UrlCow<'static>>,
}

impl From<ActionCow> for UploadOpt {
	fn from(mut a: ActionCow) -> Self { Self { urls: a.take_seq() } }
}

impl FromLua for UploadOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UploadOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
