use ratatui::{buffer::Buffer, layout::{self, Constraint, Direction, Rect}, style::{Color, Style}, widgets::{Paragraph, Widget}};

use super::tabs::Tabs;
use crate::{misc::readable_home, ui::Ctx};

pub struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Layout<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks = layout::Layout::new()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(area);

		let current = &self.cx.manager.current();
		let location = if current.in_search {
			format!("{} (search)", readable_home(&current.cwd))
		} else {
			format!("{}", readable_home(&current.cwd))
		};

		Paragraph::new(location).style(Style::new().fg(Color::Cyan)).render(chunks[0], buf);

		Tabs::new(self.cx).render(chunks[1], buf);
	}
}
