use std::ops::Deref;

use mlua::{FromLua, Lua, Table, UserData, UserDataFields};

#[derive(Clone, Copy, FromLua)]
pub struct Position(ratatui::layout::Position);

impl Deref for Position {
	type Target = ratatui::layout::Position;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<ratatui::layout::Rect> for Position {
	fn from(rect: ratatui::layout::Rect) -> Self { Self(rect.as_position()) }
}

impl Position {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, args): (Table, Table)| {
			Ok(Self(ratatui::layout::Position { x: args.raw_get("x")?, y: args.raw_get("y")? }))
		})?;

		let position = lua.create_table_from([("default", Self(Default::default()))])?;

		position.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Position", position)
	}
}

impl UserData for Position {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, me| Ok(me.x));
		fields.add_field_method_get("y", |_, me| Ok(me.y));
	}
}
