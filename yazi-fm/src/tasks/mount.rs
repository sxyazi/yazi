use ratatui::{
	buffer::Buffer,
	layout::{self, Alignment, Constraint, Rect},
	text::Line,
	widgets::{Block, BorderType, List, ListItem, Padding, Widget},
};
use yazi_config::THEME;
use yazi_core::tasks::TASKS_PERCENT;

use crate::Ctx;

pub(crate) struct Mount<'a> {
	cx: &'a Ctx,
}

impl<'a> Mount<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self {
		Self { cx }
	}

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

impl Widget for Mount<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let area = Self::area(area);

		yazi_plugin::elements::Clear::default().render(area, buf);
		let block = Block::bordered()
			.title(Line::styled("Mount", THEME.tasks.title))
			.title_alignment(Alignment::Center)
			.padding(Padding::symmetric(1, 1))
			.border_type(BorderType::Rounded)
			.border_style(THEME.tasks.border);
		block.clone().render(area, buf);

		let mnt = &self.cx.mount;
		let items = mnt
			.points
			.iter()
			.enumerate()
			.map(|(i, p)| {
				let mut item = ListItem::new(format!("{} {}", p.dev, p.path.to_string_lossy()));
				if i == mnt.cursor {
					item = item.style(THEME.tasks.hovered);
				}
				item
			})
			.collect::<Vec<_>>();

		List::new(items).render(block.inner(area), buf);
	}
}
