use mlua::{FromLua, Lua, Value};

pub enum FetchState {
	Bool(bool),
	Vec(Vec<bool>),
}

impl FetchState {
	pub fn get(&self, idx: usize) -> bool {
		match self {
			Self::Bool(b) => *b,
			Self::Vec(v) => v.get(idx).copied().unwrap_or(false),
		}
	}
}

impl FromLua for FetchState {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::Boolean(b) => Self::Bool(b),
			Value::Table(tbl) => Self::Vec(tbl.sequence_values().collect::<mlua::Result<_>>()?),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "FetchState".to_owned(),
				message: Some("expected a boolean or a table of booleans".to_owned()),
			})?,
		})
	}
}
