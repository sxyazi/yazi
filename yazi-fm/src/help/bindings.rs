use ratatui::{buffer::Buffer, layout::{self, Constraint, Rect}, widgets::{List, ListItem, Widget}};
use yazi_config::THEME;

use crate::Ctx;

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

		// On
		let col1: Vec<_> =
			bindings.iter().map(|c| ListItem::new(c.on()).style(THEME.help.on)).collect();

		// Run
		let col2: Vec<_> =
			bindings.iter().map(|c| ListItem::new(c.run()).style(THEME.help.run)).collect();

		// Desc
		let col3: Vec<_> = bindings
			.iter()
			.map(|c| ListItem::new(c.desc().unwrap_or("-".into())).style(THEME.help.desc))
			.collect();

		let chunks = layout::Layout::horizontal([
			Constraint::Ratio(2, 10),
			Constraint::Ratio(3, 10),
			Constraint::Ratio(5, 10),
		])
		.split(area);

		let cursor = self.cx.help.rel_cursor() as u16;
		buf.set_style(
			Rect { x: area.x, y: area.y + cursor, width: area.width, height: 1 },
			THEME.help.hovered,
		);

		List::new(col1).render(chunks[0], buf);
		List::new(col2).render(chunks[1], buf);
		List::new(col3).render(chunks[2], buf);
	}
}
