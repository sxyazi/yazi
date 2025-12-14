use std::ffi::OsString;

use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{CompletionToken, event::CmdCow, url::UrlCow};

// --- Exec
#[derive(Debug)]
pub struct ProcessOpenOpt {
	pub cwd:    UrlCow<'static>,
	pub cmd:    OsString,
	pub args:   Vec<UrlCow<'static>>,
	pub block:  bool,
	pub orphan: bool,
	pub done:   Option<CompletionToken>,

	pub spread: bool, // TODO: remove
}

impl TryFrom<CmdCow> for ProcessOpenOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		c.take_any("opt").ok_or_else(|| anyhow!("Missing 'opt' in ProcessOpenOpt"))
	}
}

impl FromLua for ProcessOpenOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ProcessOpenOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
