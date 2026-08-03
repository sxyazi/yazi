use std::rc::Rc;

use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, UserData, UserDataMethods, Value};
use ratatui_core::{buffer::Buffer, layout::{Layout, Rect}, widgets::{StatefulWidget, Widget}};

use super::{Area, Row};
use crate::{elements::{Constraint, Spatial}, style::Style};

// --- Table
#[derive(Clone, Debug, Default)]
pub struct Table {
	area: Area,

	rows:           Vec<Row>,
	header:         Option<ratatui_widgets::table::Row<'static>>,
	footer:         Option<ratatui_widgets::table::Row<'static>>,
	widths:         Vec<ratatui_core::layout::Constraint>,
	column_spacing: u16,
	block:          Option<ratatui_widgets::block::Block<'static>>, // TODO

	style:                  ratatui_core::style::Style,
	row_highlight_style:    ratatui_core::style::Style,
	column_highlight_style: ratatui_core::style::Style,
	cell_highlight_style:   ratatui_core::style::Style,

	highlight_symbol:  ratatui_core::text::Text<'static>, // TODO
	highlight_spacing: ratatui_widgets::table::HighlightSpacing, // TODO

	flex: ratatui_core::layout::Flex,

	state: ratatui_widgets::table::TableState,
}

impl Table {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, rows): (mlua::Table, Vec<Row>)| {
			Ok(Self { rows, ..Default::default() })
		})?;

		let table = lua.create_table()?;
		table.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		table.into_lua(lua)
	}

	pub fn selected_cell(&self) -> Option<&ratatui_core::text::Text<'_>> {
		let row = &self.rows[self.selected()?];
		let col = self.state.selected_column()?;
		if row.cells.is_empty() {
			None
		} else {
			Some(&row.cells[col.min(row.cells.len() - 1)].text.inner)
		}
	}

	pub fn len(&self) -> usize { self.rows.len() }

	pub fn select(&mut self, idx: Option<usize>) {
		self
			.state
			.select(idx.map(|i| if self.rows.is_empty() { 0 } else { i.min(self.rows.len() - 1) }));
	}

	pub fn selected(&self) -> Option<usize> {
		if self.rows.is_empty() { None } else { Some(self.state.selected()?.min(self.rows.len() - 1)) }
	}

	fn render_overlays(
		area: Rect,
		buf: &mut Buffer,
		rows: &[Row],
		columns: &[Rect],
		row_offset: usize,
	) {
		let mut y_offset = 0;
		for row in rows.iter().skip(row_offset) {
			if y_offset >= area.height {
				break;
			}

			row.render_overlays(Rect { y: area.y.saturating_add(y_offset), ..area }, buf, columns);
			y_offset = y_offset.saturating_add(row.height_with_margin());
		}
	}

	fn column_widths(&self, width: u16) -> Rc<[Rect]> {
		let count =
			self.rows.iter().map(|row| row.cells.len()).max().unwrap_or_default().max(self.widths.len());

		if self.widths.is_empty() {
			Layout::horizontal(vec![
				ratatui_core::layout::Constraint::Length(width / count.max(1) as u16);
				count
			])
		} else {
			Layout::horizontal(&self.widths)
		}
		.flex(self.flex)
		.spacing(self.column_spacing)
		.split(Rect::new(0, 0, width, 1))
	}
}

impl TryFrom<&AnyUserData> for Table {
	type Error = mlua::Error;

	fn try_from(value: &AnyUserData) -> Result<Self, Self::Error> { value.take() }
}

impl Spatial for Table {
	fn area(&self) -> Area { self.area }

	fn set_area(&mut self, area: Area) { self.area = area; }
}

impl Widget for Table {
	fn render(mut self, rect: ratatui_core::layout::Rect, buf: &mut Buffer)
	where
		Self: Sized,
	{
		let columns = self.column_widths(rect.width);
		for row in &mut self.rows {
			row.measure(&columns);
		}

		let mut table = ratatui_widgets::table::Table::new(self.rows.clone(), self.widths)
			.column_spacing(self.column_spacing)
			.style(self.style)
			.row_highlight_style(self.row_highlight_style)
			.column_highlight_style(self.column_highlight_style)
			.cell_highlight_style(self.cell_highlight_style)
			.highlight_symbol(self.highlight_symbol)
			.highlight_spacing(self.highlight_spacing)
			.flex(self.flex);

		if let Some(header) = self.header {
			table = table.header(header);
		}
		if let Some(footer) = self.footer {
			table = table.footer(footer);
		}
		if let Some(block) = self.block {
			table = table.block(block);
		}

		let mut state = self.state;
		StatefulWidget::render(table, rect, buf, &mut state);
		Self::render_overlays(rect, buf, &self.rows, &columns, state.offset());
	}
}

impl Widget for &Table {
	fn render(self, rect: ratatui_core::layout::Rect, buf: &mut Buffer)
	where
		Self: Sized,
	{
		self.clone().render(rect, buf);
	}
}

impl UserData for Table {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);

		methods.add_function("header", |_, (ud, header): (AnyUserData, Row)| {
			ud.borrow_mut::<Self>()?.header = Some(header.into());
			Ok(ud)
		});
		methods.add_function("footer", |_, (ud, footer): (AnyUserData, Row)| {
			ud.borrow_mut::<Self>()?.footer = Some(footer.into());
			Ok(ud)
		});
		methods.add_function("widths", |_, (ud, widths): (AnyUserData, Vec<Constraint>)| {
			ud.borrow_mut::<Self>()?.widths = widths.into_iter().map(Into::into).collect();
			Ok(ud)
		});
		methods.add_function("spacing", |_, (ud, spacing): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.column_spacing = spacing;
			Ok(ud)
		});

		methods.add_function("row", |_, (ud, idx): (AnyUserData, Option<usize>)| {
			ud.borrow_mut::<Self>()?.state.select(idx);
			Ok(ud)
		});
		methods.add_function("col", |_, (ud, idx): (AnyUserData, Option<usize>)| {
			ud.borrow_mut::<Self>()?.state.select_column(idx);
			Ok(ud)
		});

		methods.add_function("style", |_, (ud, style): (AnyUserData, Style)| {
			ud.borrow_mut::<Self>()?.style = style.0;
			Ok(ud)
		});
		methods.add_function("row_style", |_, (ud, style): (AnyUserData, Style)| {
			ud.borrow_mut::<Self>()?.row_highlight_style = style.0;
			Ok(ud)
		});
		methods.add_function("col_style", |_, (ud, style): (AnyUserData, Style)| {
			ud.borrow_mut::<Self>()?.column_highlight_style = style.0;
			Ok(ud)
		});
		methods.add_function("cell_style", |_, (ud, style): (AnyUserData, Style)| {
			ud.borrow_mut::<Self>()?.cell_highlight_style = style.0;
			Ok(ud)
		});
	}
}
