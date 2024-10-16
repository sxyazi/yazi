use ratatui::{
	buffer::Buffer,
	layout::{Margin, Rect},
	widgets::{Block, Borders, Paragraph, Widget},
};

use yazi_config::THEME;

pub(crate) struct Content<'a> {
	p: Paragraph<'a>,
}

impl<'a> Content<'a> {
	pub(crate) fn new(p: Paragraph<'a>) -> Self {
		Self { p }
	}
}

impl<'a> Widget for Content<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		// Content area
		let inner = area.inner(Margin::new(1, 0));

		// Bottom border
		let mut block = Block::new();
		if THEME.confirm.show_separators {
			block = block.borders(Borders::BOTTOM).border_style(THEME.confirm.border);
		}
		block.clone().render(area.inner(Margin::new(1, 0)), buf);

		self
			.p
			.alignment(ratatui::layout::Alignment::Center)
			.block(block)
			.style(THEME.confirm.content)
			.render(inner, buf);
	}
}
