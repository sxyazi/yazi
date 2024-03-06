use mlua::{AnyUserData, ExternalError, Lua, Table, UserData, Value};
use ratatui::widgets::{Borders, Widget};

use super::{RectRef, Renderable, Style};

// Type
const PLAIN: u8 = 0;
const ROUNDED: u8 = 1;
const DOUBLE: u8 = 2;
const THICK: u8 = 3;
const QUADRANT_INSIDE: u8 = 4;
const QUADRANT_OUTSIDE: u8 = 5;

#[derive(Clone, Default)]
pub struct Border {
	area: ratatui::layout::Rect,

	position: ratatui::widgets::Borders,
	type_:    ratatui::widgets::BorderType,
	style:    Option<ratatui::style::Style>,
}

impl Border {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, area, position): (Table, RectRef, u8)| {
			Ok(Border {
				area: *area,
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
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("type", |_, (ud, value): (AnyUserData, u8)| {
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
		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				match value {
					Value::Nil => me.style = None,
					Value::Table(tb) => me.style = Some(Style::try_from(tb)?.0),
					Value::UserData(ud) => me.style = Some(ud.borrow::<Style>()?.0),
					_ => return Err("expected a Style or Table or nil".into_lua_err()),
				}
			}
			Ok(ud)
		});
	}
}

impl Renderable for Border {
	fn area(&self) -> ratatui::layout::Rect { self.area }

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		let mut block =
			ratatui::widgets::Block::default().borders(self.position).border_type(self.type_);

		if let Some(style) = self.style {
			block = block.border_style(style);
		}

		block.render(self.area, buf);
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf); }
}
