use mlua::{ExternalError, FromLua, Lua, Table, Value};
use ratatui::style::Style;

#[derive(Clone, Debug, Default)]
pub struct InputStyles {
	pub normal:   Option<Style>,
	pub selected: Option<Style>,
}

impl TryFrom<Table> for InputStyles {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		Ok(Self {
			normal:   t.raw_get::<Option<yazi_binding::style::Style>>("normal")?.map(Into::into),
			selected: t.raw_get::<Option<yazi_binding::style::Style>>("selected")?.map(Into::into),
		})
	}
}

impl FromLua for InputStyles {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(Self::default()),
			Value::Table(t) => Self::try_from(t),
			_ => Err("expected a table for InputStyles".into_lua_err()),
		}
	}
}
