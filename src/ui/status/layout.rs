use ratatui::{buffer::Buffer, layout::{self, Constraint, Direction, Rect}, style::{Color, Modifier, Style}, widgets::{Paragraph, Widget}};

use super::Progress;
use crate::ui::Ctx;

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

		Paragraph::new("").style(Style::default().fg(mode.color().bg_rgb())).render(chunks[0], buf);

		Paragraph::new(format!(" {} ", mode))
			.style(
				Style::default()
					.fg(mode.color().fg_rgb())
					.bg(mode.color().bg_rgb())
					.add_modifier(Modifier::BOLD),
			)
			.render(chunks[1], buf);

		Paragraph::new(" master ")
			.style(Style::default().fg(mode.color().bg_rgb()).bg(Color::Rgb(72, 77, 102)))
			.render(chunks[2], buf);

		Paragraph::new("").style(Style::default().fg(Color::Rgb(72, 77, 102))).render(chunks[3], buf);

		Progress::new(self.cx).render(chunks[4], buf);
	}
}
