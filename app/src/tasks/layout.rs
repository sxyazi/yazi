use core::{tasks::TASKS_PERCENT, Ctx};

use config::THEME;
use ratatui::{buffer::Buffer, layout::{self, Alignment, Constraint, Direction, Rect}, widgets::{Block, BorderType, Borders, List, ListItem, Padding, Widget}};

use super::Clear;

pub(crate) struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }

	pub(super) fn area(area: Rect) -> Rect {
		let chunk = layout::Layout::new()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Percentage((100 - TASKS_PERCENT) / 2),
				Constraint::Percentage(TASKS_PERCENT),
				Constraint::Percentage((100 - TASKS_PERCENT) / 2),
			])
			.split(area)[1];

		layout::Layout::new()
			.direction(Direction::Horizontal)
			.constraints([
				Constraint::Percentage((100 - TASKS_PERCENT) / 2),
				Constraint::Percentage(TASKS_PERCENT),
				Constraint::Percentage((100 - TASKS_PERCENT) / 2),
			])
			.split(chunk)[1]
	}
}

impl<'a> Widget for Layout<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let area = Self::area(area);

		Clear.render(area, buf);
		let block = Block::new()
			.title("Tasks")
			.title_alignment(Alignment::Center)
			.padding(Padding::new(0, 0, 1, 1))
            // Maybe also add these border type in to the later theme system
			.borders(Borders::ALL)
			.border_type(BorderType::Rounded)
			.border_style(THEME.tasks.border.get());
		block.clone().render(area, buf);

		let tasks = &self.cx.tasks;
		let items = tasks
			.paginate()
			.iter()
			.enumerate()
			.map(|(i, v)| {
				let mut item = ListItem::new(v.name.clone());
				if i == tasks.cursor {
					item = item.style(THEME.tasks.items.get());
				}
				item
			})
			.collect::<Vec<_>>();

		List::new(items).render(block.inner(area), buf);
	}
}
