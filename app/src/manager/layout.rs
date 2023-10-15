use core::Ctx;

use config::{MANAGER, THEME};
use plugin::layout::Bar;
use ratatui::{buffer::Buffer, layout::{self, Constraint, Direction, Rect}, widgets::{Block, Borders, Padding, Widget}};

use super::{Folder, Preview};

pub(crate) struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Layout<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let layout = &MANAGER.layout;
		let manager = &self.cx.manager;

		let chunks = layout::Layout::new()
			.direction(Direction::Horizontal)
			.constraints([
				Constraint::Ratio(layout.parent, layout.all),
				Constraint::Ratio(layout.current, layout.all),
				Constraint::Ratio(layout.preview, layout.all),
			])
			.split(area);

		// Parent
		Bar::new(chunks[0], Borders::RIGHT)
			.symbol(&THEME.manager.border_symbol)
			.style(THEME.manager.border_style.into())
			.render(buf);
		if manager.parent().is_some() {
			Folder::Parent.render(Block::new().padding(Padding::new(1, 1, 0, 0)).inner(chunks[0]), buf);
		}

		// Current
		Folder::Current.render(chunks[1], buf);

		// Preview
		Bar::new(chunks[2], Borders::LEFT)
			.symbol(&THEME.manager.border_symbol)
			.style(THEME.manager.border_style.into())
			.render(buf);
		Preview::new(self.cx)
			.render(Block::new().padding(Padding::new(1, 1, 0, 0)).inner(chunks[2]), buf);
	}
}
