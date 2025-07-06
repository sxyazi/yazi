use std::ops::{Add, AddAssign, Deref};

use mlua::{FromLua, IntoLua, Lua, MetaMethod, Table, UserData, Value};

#[derive(Clone, Copy, Default, FromLua)]
pub struct Pad(ratatui::widgets::Padding);

impl Deref for Pad {
	type Target = ratatui::widgets::Padding;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<ratatui::widgets::Padding> for Pad {
	fn from(pad: ratatui::widgets::Padding) -> Self { Self(pad) }
}

impl Pad {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new =
			lua.create_function(|_, (_, top, right, bottom, left): (Table, u16, u16, u16, u16)| {
				Ok(Self(ratatui::widgets::Padding::new(left, right, top, bottom)))
			})?;

		let pad = lua.create_table_from([
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

		pad.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));
		pad.into_lua(lua)
	}
}

impl UserData for Pad {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("left", |_, me| Ok(me.left));
		fields.add_field_method_get("right", |_, me| Ok(me.right));
		fields.add_field_method_get("top", |_, me| Ok(me.top));
		fields.add_field_method_get("bottom", |_, me| Ok(me.bottom));
	}
}

impl Add<ratatui::widgets::Padding> for Pad {
	type Output = Self;

	fn add(self, rhs: ratatui::widgets::Padding) -> Self::Output {
		Self(ratatui::widgets::Padding::new(
			self.left.saturating_add(rhs.left),
			self.right.saturating_add(rhs.right),
			self.top.saturating_add(rhs.top),
			self.bottom.saturating_add(rhs.bottom),
		))
	}
}

impl AddAssign<ratatui::widgets::Padding> for Pad {
	fn add_assign(&mut self, rhs: ratatui::widgets::Padding) { *self = *self + rhs; }
}
