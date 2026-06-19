use std::ops::{Add, AddAssign, Deref};

use mlua::{FromLua, IntoLua, Lua, MetaMethod, Table, UserData, UserDataFields, Value};

#[derive(Clone, Copy, Default, FromLua)]
pub struct Pad(ratatui_widgets::block::Padding);

impl Deref for Pad {
	type Target = ratatui_widgets::block::Padding;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<ratatui_widgets::block::Padding> for Pad {
	fn from(value: ratatui_widgets::block::Padding) -> Self { Self(value) }
}

impl From<Pad> for ratatui_widgets::block::Padding {
	fn from(value: Pad) -> Self { value.0 }
}

impl Pad {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new =
			lua.create_function(|_, (_, top, right, bottom, left): (Table, u16, u16, u16, u16)| {
				Ok(Self(ratatui_widgets::block::Padding::new(left, right, top, bottom)))
			})?;

		let pad = lua.create_table_from([
			(
				"left",
				lua
					.create_function(|_, left: u16| Ok(Self(ratatui_widgets::block::Padding::left(left))))?,
			),
			(
				"right",
				lua.create_function(|_, right: u16| {
					Ok(Self(ratatui_widgets::block::Padding::right(right)))
				})?,
			),
			(
				"top",
				lua.create_function(|_, top: u16| Ok(Self(ratatui_widgets::block::Padding::top(top))))?,
			),
			(
				"bottom",
				lua.create_function(|_, bottom: u16| {
					Ok(Self(ratatui_widgets::block::Padding::bottom(bottom)))
				})?,
			),
			(
				"x",
				lua.create_function(|_, x: u16| {
					Ok(Self(ratatui_widgets::block::Padding::new(x, x, 0, 0)))
				})?,
			),
			(
				"y",
				lua.create_function(|_, y: u16| {
					Ok(Self(ratatui_widgets::block::Padding::new(0, 0, y, y)))
				})?,
			),
			(
				"xy",
				lua.create_function(|_, xy: u16| {
					Ok(Self(ratatui_widgets::block::Padding::new(xy, xy, xy, xy)))
				})?,
			),
		])?;

		pad.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;
		pad.into_lua(lua)
	}
}

impl UserData for Pad {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("left", |_, me| Ok(me.left));
		fields.add_field_method_get("right", |_, me| Ok(me.right));
		fields.add_field_method_get("top", |_, me| Ok(me.top));
		fields.add_field_method_get("bottom", |_, me| Ok(me.bottom));
	}
}

impl Add<ratatui_widgets::block::Padding> for Pad {
	type Output = Self;

	fn add(self, rhs: ratatui_widgets::block::Padding) -> Self::Output {
		Self(ratatui_widgets::block::Padding::new(
			self.left.saturating_add(rhs.left),
			self.right.saturating_add(rhs.right),
			self.top.saturating_add(rhs.top),
			self.bottom.saturating_add(rhs.bottom),
		))
	}
}

impl AddAssign<ratatui_widgets::block::Padding> for Pad {
	fn add_assign(&mut self, rhs: ratatui_widgets::block::Padding) { *self = *self + rhs; }
}
