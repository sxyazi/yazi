use ratatui::{buffer::Buffer, layout::{Margin, Rect}, widgets::{Block, BorderType, Widget}};
use yazi_config::THEME;

use crate::{Ctx, pick::List};

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

		yazi_plugin::elements::Clear::default().render(area, buf);

		Block::bordered()
			.title(pick.title())
			.border_type(BorderType::Rounded)
			.border_style(THEME.pick.border)
			.render(area, buf);

		List::new(self.cx).render(area.inner(Margin::new(0, 1)), buf);
	}
}
