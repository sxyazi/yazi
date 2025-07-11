use ratatui::{buffer::Buffer, layout::{Margin, Rect}, text::Line, widgets::{Block, BorderType, Widget}};
use yazi_config::THEME;
use yazi_core::Core;

pub(crate) struct Input<'a> {
	core: &'a Core,
}

impl<'a> Input<'a> {
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Input<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let input = &self.core.input;
		let area = self.core.mgr.area(input.position);

		yazi_binding::elements::Clear::default().render(area, buf);

		Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(THEME.input.border)
			.title(Line::styled(&input.title, THEME.input.title))
			.render(area, buf);

		input.render(area.inner(Margin::new(1, 1)), buf);
	}
}
