use ratatui_core::{buffer::Buffer, layout::{Margin, Rect}, style::Styled, widgets::Widget};
use ratatui_widgets::{block::Block, borders::Borders};
use yazi_config::THEME;
use yazi_core::Core;

pub(crate) struct Body<'a> {
	core:   &'a Core,
	border: bool,
}

impl<'a> Body<'a> {
	pub(crate) fn new(core: &'a Core, border: bool) -> Self { Self { core, border } }
}

impl Widget for Body<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let confirm = &self.core.confirm;

		// Inner area
		let inner = area.inner(Margin::new(1, 0));

		// Border
		let border_set = THEME.confirm.border_set.load();
		let block = Block::new()
			.borders(if self.border { Borders::BOTTOM } else { Borders::NONE })
			.border_set(border_set.as_set())
			.border_style(THEME.confirm.border.get());

		confirm
			.body
			.clone()
			.alignment(ratatui_core::layout::Alignment::Center)
			.block(block)
			.style(THEME.confirm.body.get().patch(Styled::style(&confirm.body)))
			.render(inner, buf);
	}
}
