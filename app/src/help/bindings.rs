use ratatui::{layout::{self, Constraint}, prelude::{Buffer, Direction, Rect}, style::{Modifier, Style}, widgets::{List, ListItem, Widget}};

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
		let cursor = self.cx.help.rel_cursor();

		let col1 = bindings
			.iter()
			.enumerate()
			.map(|(i, c)| {
				let mut x = ListItem::new(c.on.iter().map(ToString::to_string).collect::<String>());

				if i == cursor {
					x = x.style(Style::new().add_modifier(Modifier::UNDERLINED));
				}
				x
			})
			.collect::<Vec<_>>();

		let col2 = bindings
			.iter()
			.enumerate()
			.map(|(i, c)| {
				let mut x =
					ListItem::new(c.exec.iter().map(ToString::to_string).collect::<Vec<_>>().join("; "));

				if i == cursor {
					x = x.style(Style::new().add_modifier(Modifier::UNDERLINED));
				}
				x
			})
			.collect::<Vec<_>>();

		let chunks = layout::Layout::new()
			.direction(Direction::Horizontal)
			.constraints(
				[Constraint::Ratio(1, 9), Constraint::Ratio(4, 9), Constraint::Ratio(4, 9)].as_ref(),
			)
			.split(area);

		List::new(col1).render(chunks[0], buf);
		List::new(col2).render(chunks[1], buf);
	}
}
