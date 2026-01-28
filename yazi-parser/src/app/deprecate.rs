use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::CmdCow};

#[derive(Debug)]
pub struct DeprecateOpt {
	pub content: SStr,
}

impl TryFrom<CmdCow> for DeprecateOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Ok(content) = c.take("content") else {
			bail!("Invalid 'content' in DeprecateOpt");
		};

		Ok(Self { content })
	}
}

impl FromLua for DeprecateOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for DeprecateOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
