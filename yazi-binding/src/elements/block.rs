use mlua::{IntoLua, Lua, MetaMethod, Table, UserData, Value};

use super::Area;

#[derive(Clone, Debug, Default)]
pub struct Block {
	pub area:  Area,
	pub style: ratatui::style::Style,
}

impl Block {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, _: Table| Ok(Self::default()))?;

		let block = lua.create_table()?;
		block.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;
		block.into_lua(lua)
	}

	pub(super) fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
		// Only render if there's actually a background color set
		if let Some(bg) = self.style.bg {
			for y in rect.top()..rect.bottom() {
				for x in rect.left()..rect.right() {
					buf[(x, y)].set_bg(bg);
				}
			}
		}
	}
}

impl UserData for Block {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, style);
		crate::impl_style_shorthands!(methods, style);
	}
}
