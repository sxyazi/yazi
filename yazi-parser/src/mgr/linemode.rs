use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{SStr, event::ActionCow};

#[derive(Debug, Deserialize)]
pub struct LinemodeForm {
	#[serde(alias = "0")]
	pub new: SStr,
}

impl TryFrom<ActionCow> for LinemodeForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> {
		let me: Self = a.deserialize()?;

		if me.new.is_empty() || me.new.len() > 20 {
			bail!("Linemode must be between 1 and 20 characters long");
		}

		Ok(me)
	}
}

impl FromLua for LinemodeForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for LinemodeForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
