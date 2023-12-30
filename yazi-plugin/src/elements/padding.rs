use mlua::{AnyUserData, Lua, Table, UserDataFields, UserDataRef};

use crate::bindings::Cast;

pub type PaddingRef<'lua> = UserDataRef<'lua, ratatui::widgets::Padding>;

pub struct Padding;

impl Padding {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|lua, args: (Table, u16, u16, u16, u16)| {
			Self::cast(lua, ratatui::widgets::Padding::new(args.1, args.2, args.3, args.4))
		})?;

		let padding = lua.create_table_from([
			(
				"left",
				lua.create_function(|lua, left: u16| {
					Self::cast(lua, ratatui::widgets::Padding::new(left, 0, 0, 0))
				})?,
			),
			(
				"right",
				lua.create_function(|lua, right: u16| {
					Self::cast(lua, ratatui::widgets::Padding::new(0, right, 0, 0))
				})?,
			),
			(
				"top",
				lua.create_function(|lua, top: u16| {
					Self::cast(lua, ratatui::widgets::Padding::new(0, 0, top, 0))
				})?,
			),
			(
				"bottom",
				lua.create_function(|lua, bottom: u16| {
					Self::cast(lua, ratatui::widgets::Padding::new(0, 0, 0, bottom))
				})?,
			),
			(
				"x",
				lua.create_function(|lua, x: u16| {
					Self::cast(lua, ratatui::widgets::Padding::new(x, x, 0, 0))
				})?,
			),
			(
				"y",
				lua.create_function(|lua, y: u16| {
					Self::cast(lua, ratatui::widgets::Padding::new(0, 0, y, y))
				})?,
			),
			(
				"xy",
				lua.create_function(|lua, xy: u16| {
					Self::cast(lua, ratatui::widgets::Padding::new(xy, xy, xy, xy))
				})?,
			),
		])?;

		padding.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.set("Padding", padding)
	}

	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<ratatui::widgets::Padding>(|reg| {
			reg.add_field_method_get("left", |_, me| Ok(me.left));
			reg.add_field_method_get("right", |_, me| Ok(me.right));
			reg.add_field_method_get("top", |_, me| Ok(me.top));
			reg.add_field_method_get("bottom", |_, me| Ok(me.bottom));
		})
	}
}

impl Cast<ratatui::widgets::Padding> for Padding {
	fn cast(lua: &Lua, data: ratatui::widgets::Padding) -> mlua::Result<AnyUserData> {
		lua.create_any_userdata(data)
	}
}
