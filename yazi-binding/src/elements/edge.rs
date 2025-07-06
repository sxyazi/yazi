use std::ops::Deref;

use mlua::{FromLua, IntoLua, Lua, Value};
use ratatui::widgets::Borders;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Edge(pub Borders);

impl Deref for Edge {
	type Target = Borders;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Edge {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("NONE", Borders::NONE.bits()),
				("TOP", Borders::TOP.bits()),
				("RIGHT", Borders::RIGHT.bits()),
				("BOTTOM", Borders::BOTTOM.bits()),
				("LEFT", Borders::LEFT.bits()),
				("ALL", Borders::ALL.bits()),
			])?
			.into_lua(lua)
	}
}

impl FromLua for Edge {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		let Value::Integer(n) = value else {
			return Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Edge".to_string(),
				message: Some("expected an integer representation of an Edge".to_string()),
			});
		};
		let Ok(Some(b)) = u8::try_from(n).map(Borders::from_bits) else {
			return Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Edge".to_string(),
				message: Some("invalid bits for Edge".to_string()),
			});
		};
		Ok(Self(b))
	}
}
