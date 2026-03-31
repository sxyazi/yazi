use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{Layer, event::ActionCow};

#[derive(Debug)]
pub struct ToggleForm {
	pub layer: Layer,
}

impl TryFrom<ActionCow> for ToggleForm {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(Self { layer: a.str(0).parse()? }) }
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
