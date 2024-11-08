use mlua::{AnyUserData, Lua, Table, UserData};
use ratatui::widgets::{Borders, Widget};

use super::{Rect, Renderable};

// Type
const PLAIN: u8 = 0;
const ROUNDED: u8 = 1;
const DOUBLE: u8 = 2;
const THICK: u8 = 3;
const QUADRANT_INSIDE: u8 = 4;
const QUADRANT_OUTSIDE: u8 = 5;

#[derive(Clone, Default)]
pub struct Border {
	area: Rect,

	position: ratatui::widgets::Borders,
	type_:    ratatui::widgets::BorderType,
	style:    ratatui::style::Style,
}

impl Border {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, position): (Table, u8)| {
			Ok(Border {
				position: ratatui::widgets::Borders::from_bits_truncate(position),
				..Default::default()
			})
		})?;

		let border = lua.create_table_from([
			// Position
			("NONE", Borders::NONE.bits()),
			("TOP", Borders::TOP.bits()),
			("RIGHT", Borders::RIGHT.bits()),
			("BOTTOM", Borders::BOTTOM.bits()),
			("LEFT", Borders::LEFT.bits()),
			("ALL", Borders::ALL.bits()),
			// Type
			("PLAIN", PLAIN),
			("ROUNDED", ROUNDED),
			("DOUBLE", DOUBLE),
			("THICK", THICK),
			("QUADRANT_INSIDE", QUADRANT_INSIDE),
			("QUADRANT_OUTSIDE", QUADRANT_OUTSIDE),
		])?;

		border.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Border", border)
	}
}

impl UserData for Border {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, style);

		methods.add_function_mut("position", |_, (ud, position): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.position = ratatui::widgets::Borders::from_bits_truncate(position);
			Ok(ud)
		});
		methods.add_function_mut("type", |_, (ud, value): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.type_ = match value {
				ROUNDED => ratatui::widgets::BorderType::Rounded,
				DOUBLE => ratatui::widgets::BorderType::Double,
				THICK => ratatui::widgets::BorderType::Thick,
				QUADRANT_INSIDE => ratatui::widgets::BorderType::QuadrantInside,
				QUADRANT_OUTSIDE => ratatui::widgets::BorderType::QuadrantOutside,
				_ => ratatui::widgets::BorderType::Plain,
			};
			Ok(ud)
		});
	}
}

impl Renderable for Border {
	fn area(&self) -> ratatui::layout::Rect { *self.area }

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		ratatui::widgets::Block::default()
			.borders(self.position)
			.border_type(self.type_)
			.border_style(self.style)
			.render(*self.area, buf);
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf); }
}
