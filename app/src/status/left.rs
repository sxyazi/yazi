use config::THEME;
use ratatui::{buffer::Buffer, layout::Rect, style::Modifier, text::{Line, Span}, widgets::{Paragraph, Widget}};
use shared::readable_size;

use crate::Ctx;

pub(super) struct Left<'a> {
	cx: &'a Ctx,
}

impl<'a> Left<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Left<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let manager = self.cx.manager.current();
		let mode = self.cx.manager.active().mode();

		// Colors
		let primary = mode.color(&THEME.status.primary);
		let secondary = mode.color(&THEME.status.secondary);
		let body = mode.color(&THEME.status.body);

		let mut spans = vec![];

		// Mode
		spans.push(Span::styled("", primary.fg()));
		spans.push(Span::styled(
			format!(" {} ", mode),
			primary.bg().fg(**secondary).add_modifier(Modifier::BOLD),
		));

		if let Some(h) = &manager.hovered {
			// Length
			if let Some(len) = h.length {
				spans.push(Span::styled(format!(" {} ", readable_size(len)), body.bg().fg(**primary)));
				spans.push(Span::styled("", body.fg()));
			}

			// Filename
			spans.push(Span::raw(format!(" {} ", h.name().unwrap())));
		}

		Paragraph::new(Line::from(spans)).render(area, buf);
	}
}
