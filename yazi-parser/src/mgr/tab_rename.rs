use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::ActionCow};

#[derive(Debug)]
pub struct TabRenameForm {
	pub name:        Option<SStr>,
	pub interactive: bool,
}

impl TryFrom<ActionCow> for TabRenameForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let name = a.take_first().ok();
		let interactive = a.bool("interactive");

		if name.is_none() && !interactive {
			bail!("either name or interactive must be specified in TabRenameForm");
		}

		Ok(Self { name, interactive })
	}
}

impl FromLua for TabRenameForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabRenameForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
