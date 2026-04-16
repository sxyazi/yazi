use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlBuf};

#[derive(Debug, Default)]
pub struct UploadForm {
	pub urls: Vec<UrlBuf>,
}

impl From<ActionCow> for UploadForm {
	fn from(mut a: ActionCow) -> Self { Self { urls: a.take_seq() } }
}

impl FromLua for UploadForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UploadForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
