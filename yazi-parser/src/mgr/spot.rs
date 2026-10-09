use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::{ActionCow, FromAction};

#[derive(Debug, Default, Deserialize)]
pub struct SpotOpt {
	pub skip:  Option<usize>,
	#[serde(default)]
	pub force: bool,
}

impl<C> FromAction<C> for SpotOpt {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl From<usize> for SpotOpt {
	fn from(skip: usize) -> Self { Self { skip: Some(skip), ..Default::default() } }
}

impl From<bool> for SpotOpt {
	fn from(force: bool) -> Self { Self { force, ..Default::default() } }
}

impl FromLua for SpotOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SpotOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
