use std::ops::Deref;

use mlua::{ExternalError, FromLua, Lua, Table, UserData, Value};

pub struct Selector(yazi_config::Selector);

impl Deref for Selector {
	type Target = yazi_config::Selector;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Selector> for yazi_config::Selector {
	fn from(value: Selector) -> Self { value.0 }
}

impl TryFrom<&Table> for Selector {
	type Error = mlua::Error;

	fn try_from(t: &Table) -> Result<Self, Self::Error> {
		Ok(Self(yazi_config::Selector::new(
			t.raw_get::<Option<mlua::String>>("url")?.map(|s| s.to_str()?.parse()).transpose()?,
			t.raw_get::<Option<mlua::String>>("mime")?.map(|s| s.to_str()?.parse()).transpose()?,
		)?))
	}
}

impl TryFrom<Table> for Selector {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> { Self::try_from(&value) }
}

impl FromLua for Selector {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::Table(t) => t.try_into()?,
			_ => Err("expected a table of Selector".into_lua_err())?,
		})
	}
}

impl UserData for Selector {}
