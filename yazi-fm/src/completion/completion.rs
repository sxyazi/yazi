use ratatui::{buffer::Buffer, layout::Rect, widgets::{Block, BorderType, Borders, Clear, List, ListItem, Widget}};
use yazi_core::{Ctx, Position};

pub(crate) struct Completion<'a> {
	cx: &'a Ctx,
}

impl<'a> Completion<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Completion<'a> {
	fn render(self, rect: Rect, buf: &mut Buffer) {
		let items =
			self.cx.completion.items.iter().map(|x| ListItem::new(x.as_str())).collect::<Vec<_>>();

		let input_area = self.cx.area(&self.cx.input.position);
		let mut area =
			self.cx.area(&Position::Sticky(Rect { x: 1, y: 0, width: 20, height: 15 }, input_area));

		if area.y > input_area.y {
			area.y = area.y.saturating_sub(1);
		} else {
			area.y = rect.height.min(area.y + 1);
			area.height = rect.height.saturating_sub(area.y).min(area.height);
		}

		Clear.render(area, buf);
		List::new(items)
			.block(Block::new().borders(Borders::ALL).border_type(BorderType::Rounded))
			.render(area, buf);
	}
}
