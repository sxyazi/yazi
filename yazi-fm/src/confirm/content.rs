use ratatui::{buffer::Buffer, layout::{Margin, Rect}, style::{Style, Stylize}, widgets::{Block, Borders, Paragraph, Widget}};

pub(crate) struct Content<'a> {
	p: Paragraph<'a>,
}

impl<'a> Content<'a> {
	pub(crate) fn new(p: Paragraph<'a>) -> Self { Self { p } }
}

impl<'a> Widget for Content<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		// Content area
		let inner = area.inner(Margin::new(1, 0));

		// Bottom border
		let block = Block::new().borders(Borders::BOTTOM).border_style(Style::new().blue());
		block.clone().render(area.inner(Margin::new(1, 0)), buf);

		self.p.alignment(ratatui::layout::Alignment::Center).block(block).render(inner, buf);
	}
}
