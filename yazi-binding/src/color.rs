use std::str::FromStr;

use mlua::{ExternalError, ExternalResult, FromLua, Lua, UserData, Value};

#[derive(Clone, Copy, Default)]
pub struct Color(pub ratatui::style::Color);

impl FromLua for Color {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(Self(match value {
			Value::String(s) => ratatui::style::Color::from_str(&s.to_str()?).into_lua_err()?,
			Value::UserData(ud) => ud.borrow::<Self>()?.0,
			_ => Err("expected a Color".into_lua_err())?,
		}))
	}
}

impl UserData for Color {}
