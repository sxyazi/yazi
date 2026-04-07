use mlua::{IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui::widgets::Widget;

use super::Area;

#[derive(Clone, Copy, Debug, Default)]
pub struct Clear {
	pub area: Area,
}

impl Clear {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, area): (Table, Area)| Ok(Self { area }))?;

		let clear = lua.create_table()?;
		clear.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		clear.into_lua(lua)
	}
}

impl Widget for Clear {
	fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		(&self).render(rect, buf);
	}
}

impl Widget for &Clear {
	fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		yazi_widgets::Clear.render(rect, buf);
	}
}

impl UserData for Clear {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
	}
}
