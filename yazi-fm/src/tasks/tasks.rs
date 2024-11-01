use ratatui::{buffer::Buffer, layout::{self, Alignment, Constraint, Rect}, text::Line, widgets::{Block, BorderType, List, ListItem, Padding, Widget}};
use yazi_config::THEME;
use yazi_core::tasks::TASKS_PERCENT;

use crate::Ctx;

pub(crate) struct Tasks<'a> {
	cx: &'a Ctx,
}

impl<'a> Tasks<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }

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

impl<'a> Widget for Tasks<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let area = Self::area(area);

		yazi_plugin::elements::Clear::default().render(area, buf);
		let block = Block::bordered()
			.title(Line::styled("Tasks", THEME.tasks.title))
			.title_alignment(Alignment::Center)
			.padding(Padding::symmetric(1, 1))
			.border_type(BorderType::Rounded)
			.border_style(THEME.tasks.border);
		block.clone().render(area, buf);

		let tasks = &self.cx.tasks;
		let items = tasks
			.summaries
			.iter()
			.take(area.height.saturating_sub(2) as usize)
			.enumerate()
			.map(|(i, v)| {
				let mut item = ListItem::new(v.name.clone());
				if i == tasks.cursor {
					item = item.style(THEME.tasks.hovered);
				}
				item
			})
			.collect::<Vec<_>>();

		List::new(items).render(block.inner(area), buf);
	}
}
