use std::ops::Deref;

use mlua::{FromLua, IntoLua, Lua, Value};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Align(pub(super) ratatui::layout::Alignment);

impl Deref for Align {
	type Target = ratatui::layout::Alignment;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Align {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("LEFT", 0), ("CENTER", 1), ("RIGHT", 2)])?.into_lua(lua)
	}
}

impl FromLua for Align {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		let Value::Integer(n) = value else {
			return Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Align".to_string(),
				message: Some("expected an integer representation of Align".to_string()),
			});
		};
		Ok(Self(match n {
			0 => ratatui::layout::Alignment::Left,
			1 => ratatui::layout::Alignment::Center,
			2 => ratatui::layout::Alignment::Right,
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Align".to_string(),
				message: Some("invalid value for Align".to_string()),
			})?,
		}))
	}
}
