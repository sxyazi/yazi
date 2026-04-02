use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{SStr, event::ActionCow};

#[derive(Debug, Deserialize)]
pub struct TabRenameForm {
	#[serde(alias = "0")]
	pub name:        Option<SStr>,
	#[serde(default)]
	pub interactive: bool,
}

impl TryFrom<ActionCow> for TabRenameForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		let me: Self = a.deserialize()?;

		if me.name.is_none() && !me.interactive {
			bail!("either name or interactive must be specified in TabRenameForm");
		}

		Ok(me)
	}
}

impl FromLua for TabRenameForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabRenameForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
