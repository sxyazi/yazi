use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::CmdCow, url::UrlCow};

#[derive(Debug, Default)]
pub struct DownloadOpt {
	pub urls: Vec<UrlCow<'static>>,
	pub open: bool,
}

impl From<CmdCow> for DownloadOpt {
	fn from(mut c: CmdCow) -> Self { Self { urls: c.take_seq(), open: c.bool("open") } }
}

impl FromLua for DownloadOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DownloadOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
