use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shared::{Layer, event::CmdCow};

#[derive(Debug)]
pub struct ToggleOpt {
	pub layer: Layer,
}

impl TryFrom<CmdCow> for ToggleOpt {
	type Error = anyhow::Error;

	fn try_from(c: CmdCow) -> Result<Self, Self::Error> {
		let Some(layer) = c.first_str() else {
			bail!("Invalid 'layer' in Toggle");
		};

		Ok(Self { layer: layer.parse()? })
	}
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
