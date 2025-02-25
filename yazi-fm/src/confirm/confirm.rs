use ratatui::{buffer::Buffer, layout::{Alignment, Constraint, Layout, Margin, Rect}, widgets::{Block, BorderType, Widget}};
use yazi_config::THEME;

use crate::Ctx;

pub(crate) struct Confirm<'a> {
	cx: &'a Ctx,
}

impl<'a> Confirm<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Confirm<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let confirm = &self.cx.confirm;
		let area = self.cx.mgr.area(confirm.position);

		yazi_plugin::elements::Clear::default().render(area, buf);

		Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(THEME.confirm.border)
			.title(confirm.title.clone().style(THEME.confirm.title.derive(confirm.title.style)))
			.title_alignment(Alignment::Center)
			.render(area, buf);

		let content = confirm.content.clone();
		let content_border = confirm.list.line_count(area.width) != 0;
		let content_height = content.line_count(area.width) as u16;

		let chunks = Layout::vertical([
			Constraint::Length(if content_height == 0 {
				0
			} else {
				content_height.saturating_add(content_border as u16)
			}),
			Constraint::Fill(1),
			Constraint::Length(1),
		])
		.split(area.inner(Margin::new(0, 1)));

		super::Content::new(self.cx, content_border).render(chunks[0], buf);
		super::List::new(self.cx).render(chunks[1], buf);
		super::Buttons.render(chunks[2], buf);
	}
}
