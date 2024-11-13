use ratatui::{buffer::Buffer, layout::{Margin, Rect}, widgets::{Block, Borders, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap}};
use yazi_config::THEME;

use crate::Ctx;

pub(crate) struct List<'a> {
	cx: &'a Ctx,
}

impl<'a> List<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for List<'_> {
	fn render(self, mut area: Rect, buf: &mut Buffer) {
		// List content area
		let inner = area.inner(Margin::new(2, 0));

		// Bottom border
		let block = Block::new().borders(Borders::BOTTOM).border_style(THEME.confirm.border);
		block.clone().render(area.inner(Margin::new(1, 0)), buf);

		let list = self
			.cx
			.confirm
			.list
			.clone()
			.scroll((self.cx.confirm.offset as u16, 0))
			.block(block)
			.style(THEME.confirm.list)
			.wrap(Wrap { trim: false });

		// Vertical scrollbar
		let lines = list.line_count(inner.width);
		if lines >= inner.height as usize {
			area.height = area.height.saturating_sub(1);
			Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
				area,
				buf,
				&mut ScrollbarState::new(lines).position(self.cx.confirm.offset),
			);
		}

		list.render(inner, buf);
	}
}
