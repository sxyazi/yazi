use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{SStr, event::ActionCow};

#[derive(Debug)]
pub struct LinemodeForm {
	pub new: SStr,
}

impl TryFrom<ActionCow> for LinemodeForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Ok(new) = a.take_first::<SStr>() else {
			bail!("a string argument is required for LinemodeForm");
		};

		if new.is_empty() || new.len() > 20 {
			bail!("Linemode must be between 1 and 20 characters long");
		}

		Ok(Self { new })
	}
}

impl FromLua for LinemodeForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for LinemodeForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
