use std::os::unix::prelude::PermissionsExt;

use config::THEME;
use ratatui::{buffer::Buffer, layout::{Alignment, Rect}, text::{Line, Span}, widgets::{Paragraph, Widget}};
use shared::file_mode;

use super::Progress;
use crate::Ctx;

pub(super) struct Right<'a> {
	cx: &'a Ctx,
}

impl<'a> Right<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	fn permissions(&self, s: &str) -> Vec<Span> {
		// Colors
		let mode = self.cx.manager.active().mode();
		let tertiary = mode.color(&THEME.status.tertiary);
		let info = mode.color(&THEME.status.info);
		let success = mode.color(&THEME.status.success);
		let warning = mode.color(&THEME.status.warning);
		let danger = mode.color(&THEME.status.danger);

		s.chars()
			.map(|c| match c {
				'-' => Span::styled("-", tertiary.fg()),
				'r' => Span::styled("r", warning.fg()),
				'w' => Span::styled("w", danger.fg()),
				'x' | 's' | 'S' | 't' | 'T' => Span::styled(c.to_string(), info.fg()),
				_ => Span::styled(c.to_string(), success.fg()),
			})
			.collect()
	}

	fn position<'b>(&self) -> Vec<Span> {
		// Colors
		let mode = self.cx.manager.active().mode();
		let primary = mode.color(&THEME.status.primary);
		let secondary = mode.color(&THEME.status.secondary);
		let body = mode.color(&THEME.status.body);

		let cursor = self.cx.manager.current().cursor();
		let length = self.cx.manager.current().files.len();
		let percent = if cursor == 0 || length == 0 { 0 } else { (cursor + 1) * 100 / length };

		vec![
			Span::raw(" "),
			Span::styled(&THEME.status.separator.opening, body.fg()),
			Span::styled(
				if percent == 0 { "  Top ".to_string() } else { format!(" {:>3}% ", percent) },
				body.bg().fg(**primary),
			),
			Span::styled(
				format!(" {:>2}/{:<2} ", (cursor + 1).min(length), length),
				primary.bg().fg(**secondary),
			),
			Span::styled(&THEME.status.separator.closing, primary.fg()),
		]
	}
}

impl Widget for Right<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let manager = self.cx.manager.current();
		let mut spans = vec![];

		// Permissions
		if let Some(h) = &manager.hovered {
			spans.extend(self.permissions(&file_mode(h.meta.permissions().mode())))
		}

		// Position
		spans.extend(self.position());

		// Progress
		let line = Line::from(spans);
		Progress::new(self.cx).render(
			Rect {
				x:      area.x + area.width.saturating_sub(21 + line.width() as u16),
				y:      area.y,
				width:  20.min(area.width),
				height: 1,
			},
			buf,
		);

		Paragraph::new(line).alignment(Alignment::Right).render(area, buf);
	}
}
