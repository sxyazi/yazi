use ratatui::{layout::{self, Constraint}, prelude::{Buffer, Direction, Rect}, style::{Color, Style, Stylize}, widgets::{List, ListItem, Widget}};

use crate::context::Ctx;

pub(super) struct Bindings<'a> {
	cx: &'a Ctx,
}

impl<'a> Bindings<'a> {
	pub(super) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Bindings<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let bindings = &self.cx.help.window();
		if bindings.is_empty() {
			return;
		}

		let col1 = bindings
			.iter()
			.map(|c| {
				let item = ListItem::new(c.on.iter().map(ToString::to_string).collect::<String>());
				item
			})
			.collect::<Vec<_>>();

		let col2 = bindings
			.iter()
			.map(|c| {
				let item = ListItem::new(c.exec());
				item
			})
			.collect::<Vec<_>>();

		let col3 = bindings
			.iter()
			.map(|c| {
				let item = ListItem::new(if let Some(ref desc) = c.desc { desc } else { "-" });
				item
			})
			.collect::<Vec<_>>();

		let chunks = layout::Layout::new()
			.direction(Direction::Horizontal)
			.constraints(
				[Constraint::Ratio(1, 9), Constraint::Ratio(4, 9), Constraint::Ratio(4, 9)].as_ref(),
			)
			.split(area);

		let cursor = self.cx.help.rel_cursor() as u16;
		buf.set_style(
			Rect { x: area.x, y: area.y + cursor, width: area.width, height: 1 },
			Style::new().bg(Color::Blue).fg(Color::Black).bold(),
		);

		List::new(col1).render(chunks[0], buf);
		List::new(col2).render(chunks[1], buf);
		List::new(col3).render(chunks[2], buf);
	}
}
