use std::str::FromStr;

use mlua::{AnyUserData, ExternalError, ExternalResult, FromLua, IntoLua, Lua, MetaMethod, Table, UserData, UserDataFields, UserDataMethods, Value};
use yazi_shim::strum::IntoStr;

use crate::{elements::Pad, position::{Offset, Origin, Position}};

const EXPECTED: &str = "expected a Pos";

impl TryFrom<&AnyUserData> for Position {
	type Error = mlua::Error;

	fn try_from(ud: &AnyUserData) -> Result<Self, Self::Error> {
		if let Ok(pos) = ud.borrow() { Ok(*pos) } else { Err(EXPECTED.into_lua_err()) }
	}
}

impl TryFrom<Table> for Position {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		Ok(Self {
			origin:  Origin::from_str(&t.raw_get::<mlua::String>(1)?.to_str()?).into_lua_err()?,
			offset:  Offset {
				x:      t.raw_get("x").unwrap_or_default(),
				y:      t.raw_get("y").unwrap_or_default(),
				width:  t.raw_get("w").unwrap_or_default(),
				height: t.raw_get("h").unwrap_or_default(),
			},
			padding: Default::default(),
		})
	}
}

impl FromLua for Position {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(tbl) => Self::try_from(tbl),
			Value::UserData(ud) => Self::try_from(&ud),
			_ => Err(EXPECTED.into_lua_err()),
		}
	}
}

impl Position {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, t): (Table, Table)| Self::try_from(t))?;

		let pos = lua.create_table()?;
		pos.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		pos.into_lua(lua)
	}
}

impl UserData for Position {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		// TODO: cache
		fields.add_field_method_get("1", |_, me| Ok(me.origin.into_str()));
		fields.add_field_method_get("x", |_, me| Ok(me.x));
		fields.add_field_method_get("y", |_, me| Ok(me.y));
		fields.add_field_method_get("w", |_, me| Ok(me.width));
		fields.add_field_method_get("h", |_, me| Ok(me.height));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_function("pad", |_, (ud, pad): (AnyUserData, Pad)| {
			ud.borrow_mut::<Self>()?.padding = pad.into();
			Ok(ud)
		});
	}
}
