use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::spot::SpotLock;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug)]
pub struct UpdateSpottedForm {
	pub lock: SpotLock,
}

impl TryFrom<ActionCow> for UpdateSpottedForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(lock) = a.take_any("lock") else {
			bail!("Invalid 'lock' in UpdateSpottedForm");
		};

		Ok(Self { lock })
	}
}

impl FromLua for UpdateSpottedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateSpottedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
