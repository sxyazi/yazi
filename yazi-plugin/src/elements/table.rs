use mlua::{AnyUserData, FromLua, Lua, UserData};
use ratatui::widgets::StatefulWidget;

use super::{Rect, Renderable, Text};
use crate::elements::Constraint;

// --- Table
#[derive(Clone, Default)]
pub struct Table {
	area: Rect,

	rows:                Vec<ratatui::widgets::Row<'static>>,
	header:              Option<ratatui::widgets::Row<'static>>,
	footer:              Option<ratatui::widgets::Row<'static>>,
	widths:              Vec<ratatui::layout::Constraint>,
	column_spacing:      u16,
	block:               Option<ratatui::widgets::Block<'static>>,
	style:               ratatui::style::Style,
	row_highlight_style: ratatui::style::Style,
	highlight_symbol:    ratatui::text::Text<'static>,
	highlight_spacing:   ratatui::widgets::HighlightSpacing,
	flex:                ratatui::layout::Flex,

	state: ratatui::widgets::TableState,
}

impl Table {
	pub fn compose(lua: &Lua) -> mlua::Result<mlua::Table> {
		let new = lua.create_function(|_, (_, area, rows): (mlua::Table, Rect, Vec<TableRow>)| {
			Ok(Self { area, rows: rows.into_iter().map(Into::into).collect(), ..Default::default() })
		})?;

		let table = lua.create_table()?;
		table.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		Ok(table)
	}
}

impl UserData for Table {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);

		methods.add_function_mut("header", |_, (ud, row): (AnyUserData, TableRow)| {
			ud.borrow_mut::<Self>()?.header = Some(row.into());
			Ok(ud)
		});
		methods.add_function_mut("footer", |_, (ud, row): (AnyUserData, TableRow)| {
			ud.borrow_mut::<Self>()?.footer = Some(row.into());
			Ok(ud)
		});
		methods.add_function_mut("widths", |_, (ud, widths): (AnyUserData, Vec<Constraint>)| {
			ud.borrow_mut::<Self>()?.widths = widths.into_iter().map(Into::into).collect();
			Ok(ud)
		});
	}
}

impl Renderable for Table {
	fn area(&self) -> ratatui::layout::Rect { *self.area }

	fn render(mut self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		let mut table = ratatui::widgets::Table::new(self.rows, self.widths)
			.column_spacing(self.column_spacing)
			.style(self.style)
			.row_highlight_style(self.row_highlight_style)
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

		table.render(*self.area, buf, &mut self.state);
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf) }
}

// --- TableRow
#[derive(Clone, Default, FromLua)]
pub struct TableRow {
	cells:         Vec<ratatui::widgets::Cell<'static>>,
	height:        u16,
	top_margin:    u16,
	bottom_margin: u16,
	style:         ratatui::style::Style,
}

impl TableRow {
	pub fn compose(lua: &Lua) -> mlua::Result<mlua::Table> {
		let new = lua.create_function(|_, (_, cols): (mlua::Table, Vec<Text>)| {
			Ok(Self { cells: cols.into_iter().map(Into::into).collect(), ..Default::default() })
		})?;

		let row = lua.create_table()?;
		row.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		Ok(row)
	}
}

impl From<TableRow> for ratatui::widgets::Row<'static> {
	fn from(value: TableRow) -> Self {
		Self::new(value.cells)
			.height(value.height)
			.top_margin(value.top_margin)
			.bottom_margin(value.bottom_margin)
			.style(value.style)
	}
}

impl UserData for TableRow {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_style_method!(methods, style);

		methods.add_function_mut("height", |_, (ud, value): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.height = value;
			Ok(ud)
		});
		methods.add_function_mut("margin_t", |_, (ud, value): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.top_margin = value;
			Ok(ud)
		});
		methods.add_function_mut("margin_b", |_, (ud, value): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.bottom_margin = value;
			Ok(ud)
		});
	}
}
