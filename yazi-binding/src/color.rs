use std::str::FromStr;

use mlua::{ExternalError, ExternalResult, UserData, Value};

#[derive(Clone, Copy, Default)]
pub struct Color(pub ratatui::style::Color);

impl TryFrom<Value> for Color {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		Ok(Self(match value {
			Value::String(s) => ratatui::style::Color::from_str(&s.to_str()?).into_lua_err()?,
			Value::UserData(ud) => ud.borrow::<Self>()?.0,
			_ => Err("expected a Color".into_lua_err())?,
		}))
	}
}

impl UserData for Color {}
