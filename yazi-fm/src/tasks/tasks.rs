use ratatui::{buffer::Buffer, layout::{self, Alignment, Constraint, Rect}, text::Line, widgets::{Block, BorderType, Widget}};
use yazi_config::THEME;
use yazi_core::{Core, tasks::TASKS_PERCENT};

use crate::tasks::List;

pub(crate) struct Tasks<'a> {
	core: &'a Core,
}

impl<'a> Tasks<'a> {
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }

	pub(super) fn area(area: Rect) -> Rect {
		let chunk = layout::Layout::vertical([
			Constraint::Percentage((100 - TASKS_PERCENT) / 2),
			Constraint::Percentage(TASKS_PERCENT),
			Constraint::Percentage((100 - TASKS_PERCENT) / 2),
		])
		.split(area)[1];

		layout::Layout::horizontal([
			Constraint::Percentage((100 - TASKS_PERCENT) / 2),
			Constraint::Percentage(TASKS_PERCENT),
			Constraint::Percentage((100 - TASKS_PERCENT) / 2),
		])
		.split(chunk)[1]
	}
}

impl Widget for Tasks<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let area = Self::area(area);

		yazi_binding::elements::Clear::default().render(area, buf);

		let block = Block::bordered()
			.title(Line::styled("Tasks", THEME.tasks.title))
			.title_alignment(Alignment::Center)
			.border_type(BorderType::Rounded)
			.border_style(THEME.tasks.border);
		(&block).render(area, buf);

		List::new(self.core).render(block.inner(area), buf);
	}
}
