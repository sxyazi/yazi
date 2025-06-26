use ratatui::{buffer::Buffer, layout::{Margin, Rect}, style::Styled, widgets::{Block, Borders, Widget}};
use yazi_config::THEME;

use crate::Ctx;

pub(crate) struct Body<'a> {
	cx:     &'a Ctx,
	border: bool,
}

impl<'a> Body<'a> {
	pub(crate) fn new(cx: &'a Ctx, border: bool) -> Self { Self { cx, border } }
}

impl Widget for Body<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let confirm = &self.cx.confirm;

		// Inner area
		let inner = area.inner(Margin::new(1, 0));

		// Border
		let block = if self.border {
			Block::new().borders(Borders::BOTTOM).border_style(THEME.confirm.border)
		} else {
			Block::new()
		};

		confirm
			.body
			.clone()
			.alignment(ratatui::layout::Alignment::Center)
			.block(block)
			.style(THEME.confirm.body.derive(Styled::style(&confirm.body)))
			.render(inner, buf);
	}
}
