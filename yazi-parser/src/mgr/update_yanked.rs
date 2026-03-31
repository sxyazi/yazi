use std::ops::Deref;

use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use yazi_shared::event::ActionCow;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateYankedOpt<'a>(pub yazi_dds::ember::EmberYank<'a>);

impl<'a> Deref for UpdateYankedOpt<'a> {
	type Target = yazi_dds::ember::EmberYank<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl TryFrom<ActionCow> for UpdateYankedOpt<'_> {
	type Error = anyhow::Error;

	fn try_from(mut a: ActionCow) -> Result<Self, Self::Error> {
		let Some(state) = a.take_any("state") else {
			bail!("Invalid 'state' in UpdateYankedOpt");
		};

		Ok(Self(state))
	}
}

impl FromLua for UpdateYankedOpt<'_> {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateYankedOpt<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { self.0.into_lua(lua) }
}
