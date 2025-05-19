use mlua::{AnyUserData, ExternalError, IntoLua, Lua, MetaMethod, UserData, Value};
use ratatui::widgets::StatefulWidget;

use super::{Area, Row};
use crate::elements::{Constraint, Style};

const EXPECTED: &str = "expected a table of Rows";

// --- Table
#[derive(Clone, Debug, Default)]
pub struct Table {
	pub(crate) area: Area,

	rows:           Vec<Row>,
	header:         Option<ratatui::widgets::Row<'static>>,
	footer:         Option<ratatui::widgets::Row<'static>>,
	widths:         Vec<ratatui::layout::Constraint>,
	column_spacing: u16,
	block:          Option<ratatui::widgets::Block<'static>>, // TODO

	style:                  ratatui::style::Style,
	row_highlight_style:    ratatui::style::Style,
	column_highlight_style: ratatui::style::Style,
	cell_highlight_style:   ratatui::style::Style,

	highlight_symbol:  ratatui::text::Text<'static>, // TODO
	highlight_spacing: ratatui::widgets::HighlightSpacing, // TODO

	flex: ratatui::layout::Flex,

	state: ratatui::widgets::TableState,
}

impl Table {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, seq): (mlua::Table, mlua::Table)| {
			let mut rows = Vec::with_capacity(seq.raw_len());
			for v in seq.sequence_values::<Value>() {
				rows.push(Row::try_from(v?).map_err(|_| EXPECTED.into_lua_err())?);
			}

			Ok(Self { rows, ..Default::default() })
		})?;

		let table = lua.create_table()?;
		table.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		table.into_lua(lua)
	}

	pub fn selected_cell(&self) -> Option<&ratatui::text::Text> {
		let row = &self.rows[self.selected()?];
		let col = self.state.selected_column()?;
		if row.cells.is_empty() { None } else { Some(&row.cells[col.min(row.cells.len() - 1)].text) }
	}

	pub(super) fn render(
		mut self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl FnOnce(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		let mut table = ratatui::widgets::Table::new(self.rows, self.widths)
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

		table.render(self.area.transform(trans), buf, &mut self.state);
	}

	#[inline]
	pub(crate) fn len(&self) -> usize { self.rows.len() }

	pub(crate) fn select(&mut self, idx: Option<usize>) {
		self
			.state
			.select(idx.map(|i| if self.rows.is_empty() { 0 } else { i.min(self.rows.len() - 1) }));
	}

	pub(crate) fn selected(&self) -> Option<usize> {
		if self.rows.is_empty() { None } else { Some(self.state.selected()?.min(self.rows.len() - 1)) }
	}
}

impl TryFrom<AnyUserData> for Table {
	type Error = mlua::Error;

	fn try_from(value: AnyUserData) -> Result<Self, Self::Error> { value.take() }
}

impl UserData for Table {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);

		methods.add_function_mut("header", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.header = Some(Row::try_from(value)?.into());
			Ok(ud)
		});
		methods.add_function_mut("footer", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.footer = Some(Row::try_from(value)?.into());
			Ok(ud)
		});
		methods.add_function_mut("widths", |_, (ud, widths): (AnyUserData, Vec<Constraint>)| {
			ud.borrow_mut::<Self>()?.widths = widths.into_iter().map(Into::into).collect();
			Ok(ud)
		});
		methods.add_function_mut("spacing", |_, (ud, spacing): (AnyUserData, u16)| {
			ud.borrow_mut::<Self>()?.column_spacing = spacing;
			Ok(ud)
		});

		methods.add_function_mut("row", |_, (ud, idx): (AnyUserData, Option<usize>)| {
			ud.borrow_mut::<Self>()?.state.select(idx);
			Ok(ud)
		});
		methods.add_function_mut("col", |_, (ud, idx): (AnyUserData, Option<usize>)| {
			ud.borrow_mut::<Self>()?.state.select_column(idx);
			Ok(ud)
		});

		methods.add_function_mut("style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.style = Style::try_from(value)?.0;
			Ok(ud)
		});
		methods.add_function_mut("row_style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.row_highlight_style = Style::try_from(value)?.0;
			Ok(ud)
		});
		methods.add_function_mut("col_style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.column_highlight_style = Style::try_from(value)?.0;
			Ok(ud)
		});
		methods.add_function_mut("cell_style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.cell_highlight_style = Style::try_from(value)?.0;
			Ok(ud)
		});
	}
}
