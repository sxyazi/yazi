use ratatui::{buffer::Buffer, layout::Rect, widgets::{Block, BorderType, List, ListItem, Widget}};
use yazi_config::THEME;

use crate::Ctx;

pub(crate) struct Pick<'a> {
	cx: &'a Ctx,
}

impl<'a> Pick<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Pick<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let pick = &self.cx.pick;
		let area = self.cx.mgr.area(pick.position);

		let items: Vec<_> = pick
			.window()
			.iter()
			.enumerate()
			.map(|(i, v)| {
				if i != pick.rel_cursor() {
					return ListItem::new(format!("  {v}")).style(THEME.pick.inactive);
				}

				ListItem::new(format!(" {v}")).style(THEME.pick.active)
			})
			.collect();

		yazi_plugin::elements::Clear::default().render(area, buf);
		List::new(items)
			.block(
				Block::bordered()
					.title(pick.title())
					.border_type(BorderType::Rounded)
					.border_style(THEME.pick.border),
			)
			.render(area, buf);
	}
}
