use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::CmdCow, url::UrlCow};

#[derive(Debug)]
pub struct ShellOpt {
	pub run: SStr,
	pub cwd: Option<UrlCow<'static>>,

	pub block:       bool,
	pub orphan:      bool,
	pub interactive: bool,

	pub cursor: Option<usize>,
}

impl TryFrom<CmdCow> for ShellOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let me = Self {
			run: c.take_first().unwrap_or_default(),
			cwd: c.take("cwd").ok(),

			block:       c.bool("block"),
			orphan:      c.bool("orphan"),
			interactive: c.bool("interactive"),

			cursor: c.get("cursor").ok(),
		};

		if me.cursor.is_some_and(|c| c > me.run.chars().count()) {
			bail!("The cursor position is out of bounds.");
		}

		Ok(me)
	}
}

impl FromLua for ShellOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShellOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
