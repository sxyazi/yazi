use std::{borrow::Cow, ffi::OsString};

use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use tokio::sync::oneshot;
use yazi_config::opener::OpenerRule;
use yazi_shared::{event::CmdCow, url::UrlBuf};

// --- Exec
#[derive(Debug)]
pub struct ProcessExecOpt {
	pub cwd:    UrlBuf,
	pub opener: Cow<'static, OpenerRule>,
	pub args:   Vec<OsString>,
	pub done:   Option<oneshot::Sender<()>>,
}

impl TryFrom<CmdCow> for ProcessExecOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		c.take_any("option").ok_or_else(|| anyhow!("Missing 'option' in ProcessExecOpt"))
	}
}

impl FromLua for ProcessExecOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ProcessExecOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
