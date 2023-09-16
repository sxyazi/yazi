use core::Ctx;

use ratatui::{layout, prelude::{Buffer, Constraint, Direction, Rect}, style::{Color, Style}, widgets::{Paragraph, Widget}};
use shared::readable_path;

use super::Tabs;

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

		let cwd = &self.cx.manager.current().cwd;
		let location = if cwd.is_search() {
			format!("{} (search: {})", readable_path(cwd), cwd.frag().unwrap())
		} else {
			readable_path(cwd)
		};

		Paragraph::new(location).style(Style::new().fg(Color::Cyan)).render(chunks[0], buf);

		Tabs::new(self.cx).render(chunks[1], buf);
	}
}
