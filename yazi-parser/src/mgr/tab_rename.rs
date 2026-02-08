use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct TabRenameOpt {
	pub name:        Option<SStr>,
	pub interactive: bool,
}

impl TryFrom<CmdCow> for TabRenameOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let name = c.take_first().ok();
		let interactive = c.bool("interactive");

		if name.is_none() && !interactive {
			bail!("either name or interactive must be specified in TabRenameOpt");
		}

		Ok(Self { name, interactive })
	}
}

impl FromLua for TabRenameOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for TabRenameOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
