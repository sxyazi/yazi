use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{event::{ActionCow, FromAction}, strand::StrandBuf};

#[derive(Debug, Deserialize)]
pub struct CreateForm {
	#[serde(alias = "0", default)]
	pub target: StrandBuf,
	#[serde(default)]
	pub dir:    bool,
	#[serde(default)]
	pub force:  bool,
}

impl<C> FromAction<C> for CreateForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl FromLua for CreateForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for CreateForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
