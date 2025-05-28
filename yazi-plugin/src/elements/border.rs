use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, Value};
use ratatui::widgets::{Borders, Widget};

use super::{Area, Edge};
use crate::elements::Line;

// Type
const PLAIN: u8 = 0;
const ROUNDED: u8 = 1;
const DOUBLE: u8 = 2;
const THICK: u8 = 3;
const QUADRANT_INSIDE: u8 = 4;
const QUADRANT_OUTSIDE: u8 = 5;

#[derive(Clone, Debug, Default)]
pub struct Border {
	pub(crate) area: Area,

	pub(crate) edge:   Edge,
	pub(crate) r#type: ratatui::widgets::BorderType,
	pub(crate) style:  ratatui::style::Style,

	pub(crate) titles: Vec<(ratatui::widgets::block::Position, ratatui::text::Line<'static>)>,
}

impl Border {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua
			.create_function(|_, (_, edge): (Table, Edge)| Ok(Border { edge, ..Default::default() }))?;

		let border = lua.create_table_from([
			// TODO: remove these constants
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

		border.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));
		border.into_lua(lua)
	}

	pub(super) fn render(
		self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl FnOnce(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		let mut block = ratatui::widgets::Block::default()
			.borders(self.edge.0)
			.border_type(self.r#type)
			.border_style(self.style);

		for title in self.titles {
			block = match title {
				(ratatui::widgets::block::Position::Top, line) => block.title(line),
				(ratatui::widgets::block::Position::Bottom, line) => block.title(line),
			};
		}

		block.render(self.area.transform(trans), buf);
	}
}

impl UserData for Border {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, style);

		methods.add_function_mut("type", |_, (ud, value): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.r#type = match value {
				ROUNDED => ratatui::widgets::BorderType::Rounded,
				DOUBLE => ratatui::widgets::BorderType::Double,
				THICK => ratatui::widgets::BorderType::Thick,
				QUADRANT_INSIDE => ratatui::widgets::BorderType::QuadrantInside,
				QUADRANT_OUTSIDE => ratatui::widgets::BorderType::QuadrantOutside,
				_ => ratatui::widgets::BorderType::Plain,
			};
			Ok(ud)
		});
		methods.add_function_mut(
			"title",
			|_, (ud, line, position): (AnyUserData, Value, Option<u8>)| {
				let position = if position == Some(Borders::BOTTOM.bits()) {
					ratatui::widgets::block::Position::Bottom
				} else {
					ratatui::widgets::block::Position::Top
				};

				ud.borrow_mut::<Self>()?.titles.push((position, Line::try_from(line)?.inner));
				Ok(ud)
			},
		);
		methods.add_function_mut("edge", |_, (ud, edge): (AnyUserData, Edge)| {
			ud.borrow_mut::<Self>()?.edge = edge;
			Ok(ud)
		});
	}
}
