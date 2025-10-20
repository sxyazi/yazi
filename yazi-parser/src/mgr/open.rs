use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{event::CmdCow, url::UrlCow};

#[derive(Debug)]
pub struct OpenOpt {
	pub cwd:         Option<UrlCow<'static>>,
	pub targets:     Vec<UrlCow<'static>>,
	pub interactive: bool,
	pub hovered:     bool,
}

impl TryFrom<CmdCow> for OpenOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		Ok(Self {
			cwd:         c.take("cwd").ok(),
			targets:     c.take_seq(),
			interactive: c.bool("interactive"),
			hovered:     c.bool("hovered"),
		})
	}
}

impl FromLua for OpenOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for OpenOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
