use ratatui::{buffer::Buffer, layout::{Margin, Rect}, style::Styled, widgets::{Block, Borders, Widget}};
use yazi_config::THEME;

use crate::Ctx;

pub(crate) struct Content<'a> {
	cx:     &'a Ctx,
	border: bool,
}

impl<'a> Content<'a> {
	pub(crate) fn new(cx: &'a Ctx, border: bool) -> Self { Self { cx, border } }
}

impl Widget for Content<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let confirm = &self.cx.confirm;

		// Content area
		let inner = area.inner(Margin::new(1, 0));

		// Border
		let block = if self.border {
			Block::new().borders(Borders::BOTTOM).border_style(THEME.confirm.border)
		} else {
			Block::new()
		};

		confirm
			.content
			.clone()
			.alignment(ratatui::layout::Alignment::Center)
			.block(block)
			.style(THEME.confirm.content.derive(Styled::style(&confirm.content)))
			.render(inner, buf);
	}
}
