use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::CmdCow, url::UrlCow};

#[derive(Debug, Default)]
pub struct OpenDoOpt {
	pub cwd:         UrlCow<'static>,
	pub targets:     Vec<UrlCow<'static>>,
	pub interactive: bool,
}

impl TryFrom<CmdCow> for OpenDoOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			opt
		} else {
			bail!("'opt' is required for OpenDoOpt");
		}
	}
}

impl FromLua for OpenDoOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for OpenDoOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
