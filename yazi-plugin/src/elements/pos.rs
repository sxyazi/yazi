use std::{ops::Deref, str::FromStr};

use mlua::{AnyUserData, ExternalResult, IntoLua, Lua, MetaMethod, Table, UserData, Value};

use super::Pad;

#[derive(Clone, Copy, Default)]
pub struct Pos {
	inner: yazi_config::popup::Position,

	pub(super) pad: Pad,
}

impl Deref for Pos {
	type Target = yazi_config::popup::Position;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<yazi_config::popup::Position> for Pos {
	fn from(value: yazi_config::popup::Position) -> Self {
		Self { inner: value, ..Default::default() }
	}
}

impl From<Pos> for yazi_config::popup::Position {
	fn from(value: Pos) -> Self { value.inner }
}

impl TryFrom<mlua::Table> for Pos {
	type Error = mlua::Error;

	fn try_from(t: mlua::Table) -> Result<Self, Self::Error> {
		use yazi_config::popup::{Offset, Origin, Position};

		Ok(Self::from(Position {
			origin: Origin::from_str(&t.raw_get::<mlua::String>(1)?.to_str()?).into_lua_err()?,
			offset: Offset {
				x:      t.raw_get("x").unwrap_or_default(),
				y:      t.raw_get("y").unwrap_or_default(),
				width:  t.raw_get("w").unwrap_or_default(),
				height: t.raw_get("h").unwrap_or_default(),
			},
		}))
	}
}

impl Pos {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, t): (Table, Table)| Self::try_from(t))?;

		let position = lua.create_table()?;
		position.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		position.into_lua(lua)
	}

	pub fn new_input(t: mlua::Table) -> mlua::Result<Self> {
		let mut p = Self::try_from(t)?;
		p.inner.offset.height = 3;
		Ok(p)
	}
}

impl UserData for Pos {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		// TODO: cache
		fields.add_field_method_get(1, |_, me| Ok(me.origin.to_string()));
		fields.add_field_method_get("x", |_, me| Ok(me.offset.x));
		fields.add_field_method_get("y", |_, me| Ok(me.offset.y));
		fields.add_field_method_get("w", |_, me| Ok(me.offset.width));
		fields.add_field_method_get("h", |_, me| Ok(me.offset.height));
	}

	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_function_mut("pad", |_, (ud, pad): (AnyUserData, Pad)| {
			ud.borrow_mut::<Self>()?.pad = pad;
			Ok(ud)
		});
	}
}
