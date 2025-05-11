use mlua::{IntoLua, Lua, LuaSerdeExt, Value};
use yazi_config::THEME;

use super::OPTS;
use crate::Composer;

pub struct Theme;

impl Theme {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, 15, |lua, key| {
			match key {
				b"mgr" => lua.to_value_with(&THEME.mgr, OPTS)?,
				b"tabs" => lua.to_value_with(&THEME.tabs, OPTS)?,
				b"mode" => lua.to_value_with(&THEME.mode, OPTS)?,
				b"status" => lua.to_value_with(&THEME.status, OPTS)?,
				b"which" => lua.to_value_with(&THEME.which, OPTS)?,
				b"confirm" => lua.to_value_with(&THEME.confirm, OPTS)?,
				b"spot" => lua.to_value_with(&THEME.spot, OPTS)?,
				b"notify" => lua.to_value_with(&THEME.notify, OPTS)?,
				b"pick" => lua.to_value_with(&THEME.pick, OPTS)?,
				b"input" => lua.to_value_with(&THEME.input, OPTS)?,
				b"cmp" => lua.to_value_with(&THEME.cmp, OPTS)?,
				b"tasks" => lua.to_value_with(&THEME.tasks, OPTS)?,
				b"help" => lua.to_value_with(&THEME.help, OPTS)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}
}
