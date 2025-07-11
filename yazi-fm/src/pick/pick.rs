use ratatui::{buffer::Buffer, layout::{Margin, Rect}, widgets::{Block, BorderType, Widget}};
use yazi_config::THEME;
use yazi_core::Core;

use crate::pick::List;

pub(crate) struct Pick<'a> {
	core: &'a Core,
}

impl<'a> Pick<'a> {
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Pick<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let pick = &self.core.pick;
		let area = self.core.mgr.area(pick.position);

		yazi_binding::elements::Clear::default().render(area, buf);

		Block::bordered()
			.title(pick.title())
			.border_type(BorderType::Rounded)
			.border_style(THEME.pick.border)
			.render(area, buf);

		List::new(self.core).render(area.inner(Margin::new(0, 1)), buf);
	}
}
