use std::ops::Deref;

use mlua::{FromLua, Lua, Table, UserData};

#[derive(Clone, Copy, FromLua)]
pub struct Padding(ratatui::widgets::Padding);

impl Deref for Padding {
	type Target = ratatui::widgets::Padding;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Padding {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, args: (Table, u16, u16, u16, u16)| {
			Ok(Self(ratatui::widgets::Padding::new(args.1, args.2, args.3, args.4)))
		})?;

		let padding = lua.create_table_from([
			(
				"left",
				lua.create_function(|_, left: u16| Ok(Self(ratatui::widgets::Padding::left(left))))?,
			),
			(
				"right",
				lua.create_function(|_, right: u16| Ok(Self(ratatui::widgets::Padding::right(right))))?,
			),
			("top", lua.create_function(|_, top: u16| Ok(Self(ratatui::widgets::Padding::top(top))))?),
			(
				"bottom",
				lua
					.create_function(|_, bottom: u16| Ok(Self(ratatui::widgets::Padding::bottom(bottom))))?,
			),
			("x", lua.create_function(|_, x: u16| Ok(Self(ratatui::widgets::Padding::new(x, x, 0, 0))))?),
			("y", lua.create_function(|_, y: u16| Ok(Self(ratatui::widgets::Padding::new(0, 0, y, y))))?),
			(
				"xy",
				lua
					.create_function(|_, xy: u16| Ok(Self(ratatui::widgets::Padding::new(xy, xy, xy, xy))))?,
			),
		])?;

		padding.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Padding", padding)
	}
}

impl UserData for Padding {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("left", |_, me| Ok(me.left));
		fields.add_field_method_get("right", |_, me| Ok(me.right));
		fields.add_field_method_get("top", |_, me| Ok(me.top));
		fields.add_field_method_get("bottom", |_, me| Ok(me.bottom));
	}
}
