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

		let constraint = (0..completion.column_cnt)
			.map(|_| Constraint::Percentage(completion.max_width))
			.collect::<Vec<Constraint>>();
		let table = {
			let max_width = completion.max_width as usize;
			let mut table = vec![];
			let mut cur_row = vec![];
			for (idx, s) in completion.list().into_iter().enumerate() {
				if idx != 0 && idx % completion.column_cnt as usize == 0 {
					let t = mem::take(&mut cur_row);
					table.push(Row::new(t));
				}
				cur_row.push(
					Cell::from(if s.len() < max_width {
						s
					} else {
						s.split_at(max_width - 1).0.to_string() + "…"
					})
					.style(if completion.cursor() == idx {
						THEME.completion.active.into()
					} else {
						THEME.completion.inactive.into()
					}),
				);
			}
			table.push(Row::new(cur_row));
			Table::new(table)
				.block(
					Block::new()
						.borders(Borders::ALL)
						.border_type(BorderType::Double)
						.border_style(THEME.completion.border.into()),
				)
				.widths(&constraint)
		};

		Clear.render(area, buf);
		table.render(area, buf);
	}
}
