use ratatui::{buffer::Buffer, layout::{self, Constraint, Direction, Rect}, style::Modifier, widgets::{Paragraph, Widget}};

use super::Progress;
use crate::{config::THEME, ui::Ctx};

pub struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Layout<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let mode = self.cx.manager.active().mode();

		let chunks = layout::Layout::default()
			.direction(Direction::Horizontal)
			.constraints(
				[
					Constraint::Length(1),
					Constraint::Length(8),
					Constraint::Length(8),
					Constraint::Length(1),
					Constraint::Min(0),
				]
				.as_ref(),
			)
			.split(area);

		let primary = mode.color(&THEME.status.primary);
		let secondary = mode.color(&THEME.status.secondary);
		let body = mode.color(&THEME.status.body);

		Paragraph::new("").style(primary.fg()).render(chunks[0], buf);

		Paragraph::new(format!(" {} ", mode))
			.style(primary.bg().fg(**secondary).add_modifier(Modifier::BOLD))
			.render(chunks[1], buf);

		Paragraph::new(" master ").style(body.bg().fg(**primary)).render(chunks[2], buf);

		Paragraph::new("").style(body.fg()).render(chunks[3], buf);

		Progress::new(self.cx).render(chunks[4], buf);
	}
}
