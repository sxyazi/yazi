use config::THEME;
use ratatui::{buffer::Buffer, layout::{Alignment, Rect}, text::{Line, Span}, widgets::{Paragraph, Widget}};

use crate::Ctx;

pub(super) struct Tabs<'a> {
	cx: &'a Ctx,
}

impl<'a> Tabs<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Tabs<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let tabs = self.cx.manager.tabs();

		let line = Line::from(
			tabs
				.iter()
				.enumerate()
				.map(|(i, _)| {
					if i == tabs.idx() {
						Span::styled(format!(" {} ", i + 1), THEME.tab.active.get())
					} else {
						Span::styled(format!(" {} ", i + 1), THEME.tab.inactive.get())
					}
				})
				.collect::<Vec<_>>(),
		);

		Paragraph::new(line).alignment(Alignment::Right).render(area, buf);
	}
}
