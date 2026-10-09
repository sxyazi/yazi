use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_binding::Scope;
use yazi_core::tab::PreviewLock;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug)]
pub struct UpdatePeekedForm {
	pub lock:  PreviewLock,
	pub scope: Scope,
}

impl<C> FromAction<C> for UpdatePeekedForm {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self {
			lock:  a.take_any("lock").ok_or_else(|| anyhow!("Invalid 'lock' in UpdatePeekedForm"))?,
			scope: a.take_any("scope").unwrap_or_default(),
		})
	}
}

impl FromLua for UpdatePeekedForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdatePeekedForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
