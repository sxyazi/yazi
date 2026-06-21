use ratatui_core::{buffer::Buffer, layout::{Margin, Rect}, text::Line, widgets::Widget};
use ratatui_widgets::{block::Block, borders::BorderType};
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

		let outer = self.core.mgr.area(input.main.position);
		yazi_widgets::clear::Clear::default().render(outer, buf);

		Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(THEME.input.border.get())
			.title(Line::styled(&input.main.title, THEME.input.title.get()))
			.render(outer, buf);

		input.main.render(outer.inner(Margin::new(1, 1)), buf);
	}
}
