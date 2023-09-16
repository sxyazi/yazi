use core::Ctx;

use ratatui::{buffer::Buffer, layout::{self, Constraint, Direction, Rect}, text::Line, widgets::{Paragraph, Widget}};
use tracing::info;

use crate::Parser;

pub(crate) struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Layout<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks = layout::Layout::new()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(area);

		// Left::new(self.cx).render(chunks[0], buf);
		// Right::new(self.cx).render(chunks[1], buf);

		let mut spans = vec![];
		if let Ok(mode) = plugin::Status::mode(self.cx) {
			spans.extend(Parser::line(&mode).spans);
		}

		let x = plugin::Status::size(self.cx);
		if x.is_err() {
			info!("Error: {:?}", x);
			return;
		}
		if let Ok(size) = x {
			spans.extend(Parser::line(&size).spans);
		}

		let x = plugin::Status::name(self.cx);
		if x.is_err() {
			info!("Error: {:?}", x);
			return;
		}
		if let Ok(name) = x {
			spans.extend(Parser::line(&name).spans);
		}

		Paragraph::new(Line::from(spans)).render(chunks[0], buf);
	}
}
