use ratatui::{
	layout::{self, Constraint},
	prelude::{Buffer, Direction, Rect},
	style::{Color, Style, Stylize},
	widgets::{List, ListItem, Widget},
};

use crate::context::Ctx;

pub(super) struct Bindings<'a> {
	cx: &'a Ctx,
}

impl<'a> Bindings<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self {
		Self { cx }
	}
}

impl Widget for Bindings<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let bindings = &self.cx.help.window();
		if bindings.is_empty() {
			return;
		}

		// The First Column
		let keys = bindings
			.iter()
			.map(|c| ListItem::new(c.on()).style(Style::new().fg(Color::Yellow)))
			.collect::<Vec<_>>();

		// The Second Column
		let commands = bindings
			.iter()
			.map(|c| ListItem::new(c.exec()).style(Style::new().fg(Color::Cyan)))
			.collect::<Vec<_>>();

		// The Third Column
		let desc = bindings
			.iter()
			.map(|c| ListItem::new(if let Some(ref desc) = c.desc { desc } else { "-" }))
			.collect::<Vec<_>>();

		let chunks = layout::Layout::new()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Ratio(2, 10), Constraint::Ratio(3, 10), Constraint::Ratio(5, 10)])
			.split(area);

		let cursor = self.cx.help.rel_cursor() as u16;
		buf.set_style(
			Rect { x: area.x, y: area.y + cursor, width: area.width, height: 1 },
			Style::new().bg(Color::Black).bold(),
		);

		for (i, col) in [keys, commands, desc].into_iter().enumerate() {
			List::new(col).render(chunks[i], buf);
		}
	}
}
