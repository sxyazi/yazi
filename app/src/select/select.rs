use core::Ctx;

use config::THEME;
use ratatui::{buffer::Buffer, layout::Rect, widgets::{Block, BorderType, Borders, Clear, List, ListItem, Widget}};

pub(crate) struct Select<'a> {
	cx: &'a Ctx,
}

impl<'a> Select<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Select<'a> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let select = &self.cx.select;
		let area = self.cx.area(&select.position);

		let items = select
			.window()
			.iter()
			.enumerate()
			.map(|(i, v)| {
				if i != select.rel_cursor() {
					return ListItem::new(format!("  {v}")).style(THEME.select.inactive.into());
				}

				ListItem::new(format!(" {v}")).style(THEME.select.active.into())
			})
			.collect::<Vec<_>>();

		Clear.render(area, buf);
		List::new(items)
			.block(
				Block::new()
					.title(select.title())
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded)
					.border_style(THEME.select.border.into()),
			)
			.render(area, buf);
	}
}
