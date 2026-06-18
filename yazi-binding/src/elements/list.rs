use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui::widgets::Widget;

use super::{Area, Text};
use crate::elements::Spatial;

// --- List
#[derive(Clone, Debug, Default)]
pub struct List {
	area: Area,

	inner: ratatui::widgets::List<'static>,
}

impl List {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, items): (Table, Vec<Text>)| {
			Ok(Self { inner: ratatui::widgets::List::new(items), ..Default::default() })
		})?;

		let list = lua.create_table()?;
		list.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		list.into_lua(lua)
	}
}

impl TryFrom<&AnyUserData> for List {
	type Error = mlua::Error;

	fn try_from(value: &AnyUserData) -> Result<Self, Self::Error> { value.take() }
}

impl Spatial for List {
	fn area(&self) -> Area { self.area }

	fn set_area(&mut self, area: Area) { self.area = area; }
}

impl Widget for &List {
	fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		(&self.inner).render(rect, buf);
	}
}

impl UserData for List {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
	}
}
