use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{SStr, event::ActionCow, url::UrlBuf};

#[derive(Debug, Deserialize)]
pub struct ShellForm {
	#[serde(default, alias = "0")]
	pub run: SStr,
	pub cwd: Option<UrlBuf>,

	#[serde(default)]
	pub block:       bool,
	#[serde(default)]
	pub orphan:      bool,
	#[serde(default)]
	pub interactive: bool,

	pub cursor: Option<usize>,
}

impl TryFrom<ActionCow> for ShellForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		let me: Self = a.deserialize()?;

		if me.cursor.is_some_and(|c| c > me.run.chars().count()) {
			bail!("The cursor position is out of bounds.");
		}

		Ok(me)
	}
}

impl FromLua for ShellForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ShellForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
