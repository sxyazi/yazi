use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::event::ActionCow;

#[derive(Debug, Default, Deserialize)]
pub struct SpotOpt {
	pub skip: Option<usize>,
}

impl TryFrom<ActionCow> for SpotOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
}

impl From<usize> for SpotOpt {
	fn from(skip: usize) -> Self { Self { skip: Some(skip) } }
}

impl FromLua for SpotOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for SpotOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
