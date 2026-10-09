use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::spot::SpotLock;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug)]
pub struct UpdateSpottedForm {
	pub lock: SpotLock,
}

impl<C> FromAction<C> for UpdateSpottedForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			lock: a.take_any("lock").ok_or_else(|| anyhow!("Invalid 'lock' in UpdateSpottedForm"))?,
		})
	}
}

impl FromLua for UpdateSpottedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateSpottedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
