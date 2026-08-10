use mlua::{FromLua, Lua, LuaSerdeExt, Value};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CustomPool {
	#[default]
	None,
	File,
	Plugin,
	Fetch,
	Preload,
	Process,
}

impl FromLua for CustomPool {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let pool: Option<Self> = lua.from_value(value)?;
		Ok(pool.unwrap_or_default())
	}
}
