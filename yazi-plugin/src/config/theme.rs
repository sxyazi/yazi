use mlua::{IntoLua, Lua, LuaSerdeExt, Value};
use yazi_config::THEME;

use super::OPTS;
use crate::Composer;

pub struct Theme;

impl Theme {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, 5, |lua, key| {
			match key {
				b"mgr" => lua.to_value_with(&THEME.mgr, OPTS)?,
				b"mode" => lua.to_value_with(&THEME.mode, OPTS)?,
				b"status" => lua.to_value_with(&THEME.status, OPTS)?,
				b"spot" => lua.to_value_with(&THEME.spot, OPTS)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}
}
