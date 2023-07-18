use ratatui::{buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, widgets::Widget};

use super::{header, manager, status, tasks, Ctx, Input, Select};

pub struct Root<'a> {
	cx: &'a mut Ctx,
}

impl<'a> Root<'a> {
	pub fn new(cx: &'a mut Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Root<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)].as_ref())
			.split(area);

		header::Layout::new(self.cx).render(chunks[0], buf);
		manager::Layout::new(self.cx).render(chunks[1], buf);
		status::Layout::new(self.cx).render(chunks[2], buf);

		if self.cx.tasks.visible {
			tasks::Layout::new(self.cx).render(area, buf);
		}

		if self.cx.select.visible {
			Select::new(self.cx).render(area, buf);
		}

		if self.cx.input.visible {
			Input::new(self.cx).render(area, buf);
			self.cx.cursor = Some(self.cx.input.cursor());
		} else {
			self.cx.cursor = None;
		}
	}
}
