use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{event::{ActionCow, FromAction}, url::UrlBuf};
use yazi_shim::SStr;

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

impl<C> FromAction<C> for ShellForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> {
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
