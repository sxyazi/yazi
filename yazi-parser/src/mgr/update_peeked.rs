use anyhow::bail;
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
		if let Some(opt) = a.take_any2("opt") {
			return opt;
		}

		let Some(lock) = a.take_any("lock") else {
			bail!("Invalid 'lock' in UpdatePeekedForm");
		};

		Ok(Self { lock })
	}
}

impl FromLua for UpdatePeekedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdatePeekedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
