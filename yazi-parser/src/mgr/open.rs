use std::borrow::Cow;

use anyhow::anyhow;
use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_config::opener::OpenerRule;
use yazi_shared::{event::CmdCow, url::Url};

#[derive(Clone, Copy, Debug)]
pub struct OpenOpt {
	pub interactive: bool,
	pub hovered:     bool,
}

impl From<CmdCow> for OpenOpt {
	fn from(c: CmdCow) -> Self {
		Self { interactive: c.bool("interactive"), hovered: c.bool("hovered") }
	}
}

impl IntoLua for &OpenOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Do
#[derive(Debug, Default)]
pub struct OpenDoOpt {
	pub cwd:         Url,
	pub hovered:     Url,
	pub targets:     Vec<(Url, &'static str)>,
	pub interactive: bool,
}

impl From<CmdCow> for OpenDoOpt {
	fn from(mut c: CmdCow) -> Self { c.take_any("option").unwrap_or_default() }
}

impl IntoLua for &OpenDoOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}

// --- Open with
pub struct OpenWithOpt {
	pub opener:  Cow<'static, OpenerRule>,
	pub cwd:     Url,
	pub targets: Vec<Url>,
}

impl TryFrom<CmdCow> for OpenWithOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		c.take_any("option").ok_or_else(|| anyhow!("Missing 'option' in OpenWithOpt"))
	}
}
