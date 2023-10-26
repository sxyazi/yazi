use ratatui::{buffer::Buffer, layout::{Constraint, Direction, Layout, Rect}, widgets::Widget};
use yazi_core::Ctx;
use yazi_plugin::components;

use super::{completion, input, select, tasks, which};
use crate::help;

pub(super) struct Root<'a> {
	cx: &'a Ctx,
}

impl<'a> Root<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Root<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks = Layout::new()
			.direction(Direction::Vertical)
			.constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
			.split(area);

		components::Header::new(self.cx).render(chunks[0], buf);
		components::Manager::new(self.cx).render(chunks[1], buf);
		components::Status::new(self.cx).render(chunks[2], buf);

		if self.cx.tasks.visible {
			tasks::Layout::new(self.cx).render(area, buf);
		}

		if self.cx.select.visible {
			select::Select::new(self.cx).render(area, buf);
		}

		if self.cx.input.visible {
			input::Input::new(self.cx).render(area, buf);
		}

		if self.cx.input.completion.visible {
			completion::Completion::new(self.cx).render(area, buf);
		}

		if self.cx.help.visible {
			help::Layout::new(self.cx).render(area, buf);
		}

		if self.cx.which.visible {
			which::Which::new(self.cx).render(area, buf);
		}
	}
}
