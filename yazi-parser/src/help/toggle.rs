use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{Layer, event::{ActionCow, FromAction}};

#[derive(Debug, Deserialize)]
pub struct ToggleForm {
	#[serde(alias = "0")]
	pub layer: Layer,
}

impl<C> FromAction<C> for ToggleForm {
	fn from_action(a: ActionCow, _: &C) -> anyhow::Result<Self> { Ok(a.deserialize()?) }
}

impl From<Layer> for ToggleForm {
	fn from(layer: Layer) -> Self { Self { layer } }
}

impl FromLua for ToggleForm {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ToggleForm {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
