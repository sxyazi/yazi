use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui_core::widgets::Widget;
use ratatui_widgets::borders::Borders;

use super::{Area, BorderSet, BorderType, Edge};
use crate::elements::{Line, Spatial};

#[derive(Clone, Debug, Default)]
pub struct Border {
	pub area: Area,

	pub edge:  Edge,
	pub set:   BorderSet,
	pub style: ratatui_core::style::Style,
	pub merge: ratatui_core::symbols::merge::MergeStrategy,

	pub titles: Vec<(ratatui_widgets::block::TitlePosition, ratatui_core::text::Line<'static>)>,
}

impl Border {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new =
			lua.create_function(|_, (_, edge): (Table, Edge)| Ok(Self { edge, ..Default::default() }))?;

		let border = BorderType::compose(lua)?;

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
	fn render(self, rect: ratatui_core::layout::Rect, buf: &mut ratatui_core::buffer::Buffer) {
		let mut block = ratatui_widgets::block::Block::default()
			.borders(self.edge.0)
			.border_set(self.set.as_set())
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
	fn render(self, rect: ratatui_core::layout::Rect, buf: &mut ratatui_core::buffer::Buffer) {
		self.clone().render(rect, buf);
	}
}

impl UserData for Border {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, style);

		// TODO: remove
		methods.add_function("type", |lua, (ud, value): (AnyUserData, u8)| {
			crate::deprecate!(
				lua,
				"`ui.Border:type()` has been deprecated, use `ui.Border:set()` instead in your {}.\n\nSee #4439 for more info: https://github.com/sxyazi/yazi/pull/4439"
			);
			ud.borrow_mut::<Self>()?.set = BorderType::from(value).into();
			Ok(ud)
		});
		methods.add_function("set", |_, (ud, value): (AnyUserData, BorderSet)| {
			ud.borrow_mut::<Self>()?.set = value;
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
