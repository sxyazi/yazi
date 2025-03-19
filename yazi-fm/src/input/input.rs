use ratatui::{buffer::Buffer, layout::{Margin, Rect}, text::Line, widgets::{Block, BorderType, Widget}};
use yazi_config::THEME;

use crate::Ctx;

pub(crate) struct Input<'a> {
	cx: &'a Ctx,
}

impl<'a> Input<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Input<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let input = &self.cx.input;
		let area = self.cx.mgr.area(input.position);

		yazi_plugin::elements::Clear::default().render(area, buf);

		Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(THEME.input.border)
			.title(Line::styled(&input.title, THEME.input.title))
			.render(area, buf);

		input.render(area.inner(Margin::new(1, 1)), buf);
	}
}
