use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::ActionCow, url::UrlCow};

#[derive(Debug)]
pub struct ShellOpt {
	pub run: SStr,
	pub cwd: Option<UrlCow<'static>>,

	pub block:       bool,
	pub orphan:      bool,
	pub interactive: bool,

	pub cursor: Option<usize>,
}

impl TryFrom<ActionCow> for ShellOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let me = Self {
			run: a.take_first().unwrap_or_default(),
			cwd: a.take("cwd").ok(),

			block:       a.bool("block"),
			orphan:      a.bool("orphan"),
			interactive: a.bool("interactive"),

			cursor: a.get("cursor").ok(),
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
