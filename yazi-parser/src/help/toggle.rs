use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::Deserialize;
use yazi_shared::{Layer, event::ActionCow};

#[derive(Debug, Deserialize)]
pub struct ToggleForm {
	#[serde(alias = "0")]
	pub layer: Layer,
}

impl TryFrom<ActionCow> for ToggleForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(a.deserialize()?) }
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
