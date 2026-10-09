use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::{ActionCow, FromAction}, url::UrlBuf};

#[derive(Debug, Default)]
pub struct DownloadForm {
	pub urls: Vec<UrlBuf>,
	pub open: bool,
}

impl<C> FromAction<C> for DownloadForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { urls: a.take_seq(), open: a.bool("open") })
	}
}

impl FromLua for DownloadForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DownloadForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
