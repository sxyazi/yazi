use core::Ctx;
use std::ops::ControlFlow;

use config::THEME;
use ratatui::{buffer::Buffer, layout::{Alignment, Rect}, text::{Line, Span}, widgets::{Paragraph, Widget}};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(super) struct Tabs<'a> {
	cx: &'a Ctx,
}

impl<'a> Tabs<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	fn truncate(&self, name: &str) -> String {
		let mut width = 0;
		let flow =
			name.chars().try_fold(String::with_capacity(THEME.tab.max_width as usize), |mut s, c| {
				width += c.width().unwrap_or(0);
				if s.width() < THEME.tab.max_width as usize {
					s.push(c);
					ControlFlow::Continue(s)
				} else {
					ControlFlow::Break(s)
				}
			});

		match flow {
			ControlFlow::Break(s) => s,
			ControlFlow::Continue(s) => s,
		}
	}
}

impl<'a> Widget for Tabs<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let tabs = self.cx.manager.tabs();

		let line = Line::from(
			tabs
				.iter()
				.enumerate()
				.map(|(i, tab)| {
					let mut text = format!("{}", i + 1);
					if THEME.tab.max_width >= 3 {
						text.push(' ');
						text.push_str(tab.name());
						text = self.truncate(&text);
					}

					if i == tabs.idx() {
						Span::styled(format!(" {text} "), THEME.tab.active.get())
					} else {
						Span::styled(format!(" {text} "), THEME.tab.inactive.get())
					}
				})
				.collect::<Vec<_>>(),
		);

		Paragraph::new(line).alignment(Alignment::Right).render(area, buf);
	}
}
