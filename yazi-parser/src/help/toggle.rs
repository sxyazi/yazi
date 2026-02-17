use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{Layer, event::ActionCow};

#[derive(Debug)]
pub struct ToggleOpt {
	pub layer: Layer,
}

impl TryFrom<ActionCow> for ToggleOpt {
	type Error = anyhow::Error;

	fn try_from(a: ActionCow) -> Result<Self, Self::Error> { Ok(Self { layer: a.str(0).parse()? }) }
}

impl From<Layer> for ToggleOpt {
	fn from(layer: Layer) -> Self { Self { layer } }
}

impl FromLua for ToggleOpt {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for ToggleOpt {
	fn into_lua(self, _: &Lua) -> mlua::Result<Value> { Err("unsupported".into_lua_err()) }
}
