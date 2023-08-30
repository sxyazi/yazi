use ratatui::{buffer::Buffer, layout::{self, Rect}, prelude::{Constraint, Direction}, style::{Color, Style}, widgets::{Clear, Paragraph, Widget}};

use super::Bindings;
use crate::Ctx;

pub(crate) struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Layout<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks = layout::Layout::new()
			.direction(Direction::Vertical)
			.constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
			.split(area);

		Clear.render(area, buf);
		Paragraph::new("manager.help")
			.style(Style::new().fg(Color::Rgb(35, 39, 59)).bg(Color::Rgb(200, 211, 248)))
			.render(chunks[1], buf);

		Bindings::new(self.cx).render(chunks[0], buf);
	}
}
