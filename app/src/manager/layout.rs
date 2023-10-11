use core::Ctx;

use config::MANAGER;
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
		let block = Block::new().borders(Borders::RIGHT).padding(Padding::new(1, 0, 0, 0));
		if manager.parent().is_some() {
			Folder::Parent.render(block.inner(chunks[0]), buf);
		}
		block.render(chunks[0], buf);

		// Current
		Folder::Current.render(chunks[1], buf);

		// Preview
		let block = Block::new().borders(Borders::LEFT).padding(Padding::new(0, 1, 0, 0));
		Preview::new(self.cx).render(block.inner(chunks[2]), buf);
		block.render(chunks[2], buf);
	}
}
