use ratatui::{buffer::Buffer, layout::Rect, widgets::{Block, BorderType, List, ListItem, Widget}};
use yazi_config::THEME;

use crate::{widgets, Ctx};

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

		let items: Vec<_> = select
			.window()
			.iter()
			.enumerate()
			.map(|(i, v)| {
				if i != select.rel_cursor() {
					return ListItem::new(format!("  {v}")).style(THEME.select.inactive);
				}

				ListItem::new(format!(" {v}")).style(THEME.select.active)
			})
			.collect();

		widgets::Clear.render(area, buf);
		List::new(items)
			.block(
				Block::bordered()
					.title(select.title())
					.border_type(BorderType::Rounded)
					.border_style(THEME.select.border),
			)
			.render(area, buf);
	}
}
