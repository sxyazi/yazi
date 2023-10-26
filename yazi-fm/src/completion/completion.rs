use std::mem;

use ratatui::{buffer::Buffer, layout::Rect, widgets::{Block, BorderType, Borders, Clear, Row, Table, Widget}};
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

		let table = {
			let mut table = vec![];
			let mut cur_row = vec![];
			const COLUMN_CNT: usize = 4;
			for (idx, s) in completion.list().into_iter().enumerate() {
				if idx != 0 && idx % COLUMN_CNT == 0 {
					let t = mem::take(&mut cur_row);
					table.push(Row::new(t));
				}
				cur_row.push(s);
			}
			Table::new(table).block(
				Block::new()
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded)
					// todo
					.border_style(THEME.select.border.into()),
			)
		};

		Clear.render(area, buf);
		table.render(area, buf);
	}
}
