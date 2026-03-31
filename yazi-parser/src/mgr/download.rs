use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::ActionCow, url::UrlCow};

#[derive(Debug, Default)]
pub struct DownloadForm {
	pub urls: Vec<UrlCow<'static>>,
	pub open: bool,
}

impl From<ActionCow> for DownloadForm {
	fn from(mut a: ActionCow) -> Self { Self { urls: a.take_seq(), open: a.bool("open") } }
}

impl FromLua for DownloadForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DownloadForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
