use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui_core::widgets::Widget;
use ratatui_widgets::borders::Borders;

use super::{Area, Edge};
use crate::elements::{Line, Spatial};

// Type
const PLAIN: u8 = 0;
const ROUNDED: u8 = 1;
const DOUBLE: u8 = 2;
const THICK: u8 = 3;
const QUADRANT_INSIDE: u8 = 4;
const QUADRANT_OUTSIDE: u8 = 5;

#[derive(Clone, Debug, Default)]
pub struct Border {
	pub area: Area,

	pub edge:   Edge,
	pub r#type: ratatui_widgets::borders::BorderType,
	pub style:  ratatui_core::style::Style,
	pub merge:  ratatui_core::symbols::merge::MergeStrategy,

	pub titles: Vec<(ratatui_widgets::block::TitlePosition, ratatui_core::text::Line<'static>)>,
}

impl Border {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, edge): (Table, Edge)| {
			Ok(Self { edge, r#type: ratatui_widgets::borders::BorderType::Rounded, ..Default::default() })
		})?;

		let border = lua.create_table_from([
			// Type
			("PLAIN", PLAIN),
			("ROUNDED", ROUNDED),
			("DOUBLE", DOUBLE),
			("THICK", THICK),
			("QUADRANT_INSIDE", QUADRANT_INSIDE),
			("QUADRANT_OUTSIDE", QUADRANT_OUTSIDE),
		])?;

		border.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;
		border.into_lua(lua)
	}
}

impl TryFrom<&AnyUserData> for Border {
	type Error = mlua::Error;

	fn try_from(value: &AnyUserData) -> Result<Self, Self::Error> { value.take() }
}

impl Spatial for Border {
	fn area(&self) -> Area { self.area }

	fn set_area(&mut self, area: Area) { self.area = area; }
}

impl Widget for Border {
	fn render(self, rect: ratatui_core::layout::Rect, buf: &mut ratatui_core::buffer::Buffer)
	where
		Self: Sized,
	{
		let mut block = ratatui_widgets::block::Block::default()
			.borders(self.edge.0)
			.border_type(self.r#type)
			.border_style(self.style)
			.merge_borders(self.merge);

		for title in self.titles {
			block = match title {
				(ratatui_widgets::block::TitlePosition::Top, line) => block.title_top(line),
				(ratatui_widgets::block::TitlePosition::Bottom, line) => block.title_bottom(line),
			};
		}

		block.render(rect, buf);
	}
}

impl Widget for &Border {
	fn render(self, rect: ratatui_core::layout::Rect, buf: &mut ratatui_core::buffer::Buffer)
	where
		Self: Sized,
	{
		self.clone().render(rect, buf);
	}
}

impl UserData for Border {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, style);

		methods.add_function("type", |_, (ud, value): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.r#type = match value {
				ROUNDED => ratatui_widgets::borders::BorderType::Rounded,
				DOUBLE => ratatui_widgets::borders::BorderType::Double,
				THICK => ratatui_widgets::borders::BorderType::Thick,
				QUADRANT_INSIDE => ratatui_widgets::borders::BorderType::QuadrantInside,
				QUADRANT_OUTSIDE => ratatui_widgets::borders::BorderType::QuadrantOutside,
				_ => ratatui_widgets::borders::BorderType::Plain,
			};
			Ok(ud)
		});
		methods.add_function("title", |_, (ud, line, position): (AnyUserData, Line, Option<u8>)| {
			let position = if position == Some(Borders::BOTTOM.bits()) {
				ratatui_widgets::block::TitlePosition::Bottom
			} else {
				ratatui_widgets::block::TitlePosition::Top
			};

			ud.borrow_mut::<Self>()?.titles.push((position, line.inner));
			Ok(ud)
		});
		methods.add_function("edge", |_, (ud, edge): (AnyUserData, Edge)| {
			ud.borrow_mut::<Self>()?.edge = edge;
			Ok(ud)
		});
		methods.add_function("merge", |_, (ud, exact): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.merge = if exact {
				ratatui_core::symbols::merge::MergeStrategy::Exact
			} else {
				ratatui_core::symbols::merge::MergeStrategy::Fuzzy
			};
			Ok(ud)
		});
	}
}
