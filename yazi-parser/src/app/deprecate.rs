use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::ActionCow};

#[derive(Debug)]
pub struct DeprecateOpt {
	pub content: SStr,
}

impl TryFrom<ActionCow> for DeprecateOpt {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Ok(content) = a.take("content") else {
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
