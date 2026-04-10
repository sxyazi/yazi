use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_core::tab::PreviewLock;
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug)]
pub struct UpdatePeekedForm {
	pub lock: PreviewLock,
}

impl TryFrom<ActionCow> for UpdatePeekedForm {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		Ok(Self {
			lock: a.take_any("lock").ok_or_else(|| anyhow!("Invalid 'lock' in UpdatePeekedForm"))?,
		})
	}
}

impl FromLua for UpdatePeekedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdatePeekedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
