use std::ops::ControlFlow;

use config::THEME;
use ratatui::{buffer::Buffer, layout::{Alignment, Rect}, text::{Line, Span}, widgets::{Paragraph, Widget}};
use unicode_width::UnicodeWidthStr;

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
					if let Some(dir_name) = tab.current_name() {
						text.push_str(dir_name);
					}

					let threshold = THEME.tab.max_width.max(3) - 1;
					let truncated = text.chars().try_fold(String::with_capacity(threshold), |mut text, c| {
						if text.width() > threshold {
							ControlFlow::Break(text)
						} else {
							text.push(c);
							ControlFlow::Continue(text)
						}
					});
					let mut text = match truncated {
						ControlFlow::Break(text) => text,
						ControlFlow::Continue(text) => text,
					};
					if !text.ends_with(' ') {
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
