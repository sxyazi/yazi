use anyhow::bail;
use hashbrown::HashMap;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{data::{Data, DataKey}, event::ActionCow};

#[derive(Debug)]
pub struct UpdateMimesForm {
	pub updates: HashMap<DataKey, Data>,
}

impl TryFrom<ActionCow> for UpdateMimesForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Ok(updates) = a.take("updates") else {
			bail!("Invalid 'updates' in UpdateMimesForm");
		};

		Ok(Self { updates })
	}
}

impl FromLua for UpdateMimesForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateMimesForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
