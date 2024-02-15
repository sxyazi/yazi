use std::sync::atomic::AtomicBool;

use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, widgets::Widget};

use super::{completion, input, select, tasks, which};
use crate::{components, help, Ctx};

pub(super) static COLLISION: AtomicBool = AtomicBool::new(false);

pub(super) struct Root<'a> {
	cx: &'a Ctx,
}

impl<'a> Root<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Root<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks =
			Layout::vertical([Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)])
				.split(area);

		components::Header.render(chunks[0], buf);
		components::Manager.render(chunks[1], buf);
		components::Status.render(chunks[2], buf);
		components::Preview::new(self.cx).render(area, buf);

		if self.cx.tasks.visible {
			tasks::Layout::new(self.cx).render(area, buf);
		}

		if self.cx.select.visible {
			select::Select::new(self.cx).render(area, buf);
		}

		if self.cx.input.visible {
			input::Input::new(self.cx).render(area, buf);
		}

		if self.cx.help.visible {
			help::Layout::new(self.cx).render(area, buf);
		}

		if self.cx.completion.visible {
			completion::Completion::new(self.cx).render(area, buf);
		}

		if self.cx.which.visible {
			which::Which::new(self.cx).render(area, buf);
		}
	}
}
