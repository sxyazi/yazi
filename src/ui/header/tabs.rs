use ratatui::{buffer::Buffer, layout::{Alignment, Rect}, style::{Color, Style}, text::{Line, Span}, widgets::{Paragraph, Widget}};

use crate::ui::Ctx;

pub struct Tabs<'a> {
	cx: &'a Ctx,
}

impl<'a> Tabs<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Tabs<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let tabs = self.cx.manager.tabs();

		let spans = Line::from(
			tabs
				.iter()
				.enumerate()
				.map(|(i, _)| {
					if i == tabs.idx() {
						Span::styled(format!(" {} ", i + 1), Style::default().fg(Color::Black).bg(Color::Blue))
					} else {
						Span::styled(format!(" {} ", i + 1), Style::default().fg(Color::Gray).bg(Color::Black))
					}
				})
				.collect::<Vec<_>>(),
		);

		Paragraph::new(spans).alignment(Alignment::Right).render(area, buf);
	}
}
