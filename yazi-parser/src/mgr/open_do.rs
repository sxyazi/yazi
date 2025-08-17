use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::CmdCow, url::UrlBuf};

#[derive(Debug, Default)]
pub struct OpenDoOpt {
	pub cwd:         UrlBuf,
	pub hovered:     UrlBuf,
	pub targets:     Vec<(UrlBuf, &'static str)>,
	pub interactive: bool,
}

impl From<CmdCow> for OpenDoOpt {
	fn from(mut c: CmdCow) -> Self { c.take_any("option").unwrap_or_default() }
}

impl FromLua for OpenDoOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for OpenDoOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
