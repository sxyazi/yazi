use std::borrow::Cow;

use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_config::opener::OpenerRule;
use yazi_shared::{event::CmdCow, url::UrlBuf};

#[derive(Debug)]
pub struct OpenWithOpt {
	pub opener:  Cow<'static, OpenerRule>,
	pub cwd:     UrlBuf,
	pub targets: Vec<UrlBuf>,
}

impl TryFrom<CmdCow> for OpenWithOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		c.take_any("option").ok_or_else(|| anyhow!("Missing 'option' in OpenWithOpt"))
	}
}

impl FromLua for OpenWithOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for OpenWithOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
