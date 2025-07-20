use anyhow::bail;
use mlua::{ExternalError, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct LinemodeOpt {
	pub new: SStr,
}

impl TryFrom<CmdCow> for LinemodeOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(new) = c.take_first_str() else {
			bail!("a string argument is required for LinemodeOpt");
		};

		if new.is_empty() || new.len() > 20 {
			bail!("Linemode must be between 1 and 20 characters long");
		}

		Ok(Self { new })
	}
}

impl IntoLua for &LinemodeOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
