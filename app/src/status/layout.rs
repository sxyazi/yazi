use core::Ctx;

use ratatui::{buffer::Buffer, layout::{self, Constraint, Direction, Rect}, widgets::Widget};
use tracing::info;

use crate::parser::Parser;

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

		let x = plugin::Status::layout(self.cx, area);
		if x.is_err() {
			info!("{:?}", x);
			return;
		}

		if let Ok(s) = x {
			Parser::render(&s, buf);
		}
	}
}
