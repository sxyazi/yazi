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
				.map(|(i, tab)| {
					let mut text = format!(" {} ", i + 1);
					if THEME.tab.max_width > 0 {
						text
							.push_str(&tab.current_name().chars().take(THEME.tab.max_width).collect::<String>());
						text.push(' ');
					}

					if i == tabs.idx() {
						Span::styled(text, THEME.tab.active.get())
					} else {
						Span::styled(text, THEME.tab.inactive.get())
					}
				})
				.collect::<Vec<_>>(),
		);

		Paragraph::new(line).alignment(Alignment::Right).render(area, buf);
	}
}
