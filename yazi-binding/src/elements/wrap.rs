use std::ops::Deref;

use mlua::{FromLua, IntoLua, Lua, Value};

#[derive(Clone, Copy, Debug, Default)]
pub struct Wrap(pub Option<ratatui_widgets::paragraph::Wrap>);

impl Deref for Wrap {
	type Target = Option<ratatui_widgets::paragraph::Wrap>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Wrap {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("NO", 0), ("YES", 1), ("TRIM", 2)])?.into_lua(lua)
	}
}

impl From<Wrap> for Option<ratatui_widgets::paragraph::Wrap> {
	fn from(value: Wrap) -> Self { value.0 }
}

impl From<ratatui_widgets::paragraph::Wrap> for Wrap {
	fn from(value: ratatui_widgets::paragraph::Wrap) -> Self { Self(Some(value)) }
}

impl FromLua for Wrap {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		let Value::Integer(n) = value else {
			return Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Wrap".to_string(),
				message: Some("expected an integer representation of a Wrap".to_string()),
			});
		};
		Ok(Self(match n {
			0 => None,
			1 => Some(ratatui_widgets::paragraph::Wrap { trim: false }),
			2 => Some(ratatui_widgets::paragraph::Wrap { trim: true }),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Wrap".to_string(),
				message: Some("invalid value for Wrap".to_string()),
			})?,
		}))
	}
}

impl IntoLua for Wrap {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self.0 {
			None => 0.into_lua(lua),
			Some(ratatui_widgets::paragraph::Wrap { trim: false }) => 1.into_lua(lua),
			Some(ratatui_widgets::paragraph::Wrap { trim: true }) => 2.into_lua(lua),
		}
	}
}
