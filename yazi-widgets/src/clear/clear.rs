use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui::{buffer::{Buffer, CellDiffOption}, layout::Rect, widgets::Widget};
use yazi_binding::{elements::{Area, Spatial}, impl_area_method};

use crate::clear::ClearInventory;

#[derive(Clone, Copy, Debug, Default)]
pub struct Clear {
	area: Area,
}

impl Clear {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, area): (Table, Area)| Ok(Self { area }))?;

		let clear = lua.create_table()?;
		clear.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		clear.into_lua(lua)
	}
}

impl TryFrom<&AnyUserData> for Clear {
	type Error = mlua::Error;

	fn try_from(value: &AnyUserData) -> Result<Self, Self::Error> { Ok(*value.borrow()?) }
}

impl Spatial for Clear {
	fn area(&self) -> Area { self.area }

	fn set_area(&mut self, area: Area) { self.area = area; }
}

impl Widget for Clear {
	fn render(self, area: Rect, buf: &mut Buffer)
	where
		Self: Sized,
	{
		ratatui::widgets::Clear.render(area, buf);

		for inv in inventory::iter::<ClearInventory> {
			if let Some(overlap) = (inv.clear)(area) {
				for y in overlap.top()..overlap.bottom() {
					for x in overlap.left()..overlap.right() {
						buf[(x, y)].set_diff_option(CellDiffOption::AlwaysUpdate);
					}
				}
			}
		}
	}
}

impl UserData for Clear {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		impl_area_method!(methods);
	}
}
