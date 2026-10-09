use std::ops::Deref;

use anyhow::anyhow;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateYankedForm<'a>(pub yazi_dds::ember::EmberYank<'a>);

impl<'a> Deref for UpdateYankedForm<'a> {
	type Target = yazi_dds::ember::EmberYank<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<C> FromAction<C> for UpdateYankedForm<'_> {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		a.take_any(0).map(Self).ok_or_else(|| anyhow!("Invalid payload in UpdateYankedForm"))
	}
}

impl FromLua for UpdateYankedForm<'_> {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateYankedForm<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.0.into_lua(lua) }
}
