use std::mem;

use ratatui::{buffer::Buffer, layout::{Constraint, Rect}, widgets::{Block, BorderType, Borders, Cell, Clear, Row, Table, Widget}};
use yazi_config::THEME;
use yazi_core::Ctx;

pub(crate) struct Completion<'a> {
	cx: &'a Ctx,
}

impl<'a> Completion<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Completion<'a> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let completion = &self.cx.input.completion;
		let area = self.cx.area(&completion.position);

		const COLUMN_CNT: usize = 4;
		const MAX_WIDTH: usize = 20;
		let constraint =
			(0..COLUMN_CNT).map(|_| Constraint::Ratio(1, MAX_WIDTH as u32)).collect::<Vec<Constraint>>();
		let table = {
			let mut table = vec![];
			let mut cur_row = vec![];
			for (idx, s) in completion.list().into_iter().enumerate() {
				if idx != 0 && idx % COLUMN_CNT == 0 {
					let t = mem::take(&mut cur_row);
					table.push(Row::new(t));
				}
				// todo
				cur_row.push(
					Cell::from(if s.len() < MAX_WIDTH {
						s
					} else {
						s.split_at(MAX_WIDTH - 1).0.to_string() + "…"
					})
					.style(if completion.selected_cursor() == idx {
						THEME.select.active.into()
					} else {
						THEME.select.inactive.into()
					}),
				);
			}
			table.push(Row::new(cur_row));
			Table::new(table)
				.block(
					Block::new()
					.borders(Borders::ALL)
					.border_type(BorderType::Double)
					// todo
					.border_style(THEME.select.border.into()),
				)
				.widths(&constraint)
		};

		Clear.render(area, buf);
		table.render(area, buf);
	}
}
