use std::{ops::Deref, str::FromStr};

use mlua::{ExternalResult, IntoLua};

pub struct Position(yazi_config::popup::Position);

impl Deref for Position {
	type Target = yazi_config::popup::Position;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Position> for yazi_config::popup::Position {
	fn from(value: Position) -> Self { value.0 }
}

impl<'a> TryFrom<mlua::Table<'a>> for Position {
	type Error = mlua::Error;

	fn try_from(t: mlua::Table<'a>) -> Result<Self, Self::Error> {
		use yazi_config::popup::{Offset, Origin, Position};

		Ok(Self(Position {
			origin: Origin::from_str(t.raw_get::<_, mlua::String>(1)?.to_str()?).into_lua_err()?,
			offset: Offset {
				x:      t.raw_get("x").unwrap_or_default(),
				y:      t.raw_get("y").unwrap_or_default(),
				width:  t.raw_get("w").unwrap_or_default(),
				height: 3,
			},
		}))
	}
}

impl<'lua> IntoLua<'lua> for Position {
	fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value> {
		lua
			.create_table_from([
				(1.into_lua(lua)?, self.origin.to_string().into_lua(lua)?),
				("x".into_lua(lua)?, self.offset.x.into_lua(lua)?),
				("y".into_lua(lua)?, self.offset.y.into_lua(lua)?),
				("w".into_lua(lua)?, self.offset.width.into_lua(lua)?),
				("h".into_lua(lua)?, self.offset.height.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
