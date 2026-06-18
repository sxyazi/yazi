use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui::widgets::Widget;

use super::Area;
use crate::elements::Spatial;

#[derive(Clone, Copy, Debug, Default)]
pub struct Fill {
	area: Area,

	style: ratatui::style::Style,
}

impl Fill {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new =
			lua.create_function(|_, (_, area): (Table, Area)| Ok(Self { area, ..Default::default() }))?;

		let fill = lua.create_table()?;
		fill.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		fill.into_lua(lua)
	}
}

impl TryFrom<&AnyUserData> for Fill {
	type Error = mlua::Error;

	fn try_from(value: &AnyUserData) -> Result<Self, Self::Error> { Ok(*value.borrow()?) }
}

impl Spatial for Fill {
	fn area(&self) -> Area { self.area }

	fn set_area(&mut self, area: Area) { self.area = area; }
}

impl Widget for &Fill {
	fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		if self.style == Default::default() {
			return;
		}

		for pos in rect.positions() {
			buf[pos].set_style(self.style);
		}
	}
}

impl UserData for Fill {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_style_method!(methods, style);
	}
}
