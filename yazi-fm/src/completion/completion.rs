use std::path::MAIN_SEPARATOR;

use ratatui::{buffer::Buffer, layout::Rect, widgets::{Block, BorderType, List, ListItem, Widget}};
use yazi_adapter::Dimension;
use yazi_config::{popup::{Offset, Position}, THEME};

use crate::Ctx;

pub(crate) struct Completion<'a> {
	cx: &'a Ctx,
}

impl<'a> Completion<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Completion<'a> {
	fn render(self, rect: Rect, buf: &mut Buffer) {
		let items: Vec<_> = self
			.cx
			.completion
			.window()
			.iter()
			.enumerate()
			.map(|(i, x)| {
				let icon = if x.ends_with(MAIN_SEPARATOR) {
					&THEME.completion.icon_folder
				} else {
					&THEME.completion.icon_file
				};

				let mut item = ListItem::new(format!(" {icon} {x}"));
				if i == self.cx.completion.rel_cursor() {
					item = item.style(THEME.completion.active);
				} else {
					item = item.style(THEME.completion.inactive);
				}

				item
			})
			.collect();

		let input_area = self.cx.area(&self.cx.input.position);
		let mut area = Position::sticky(Dimension::available(), input_area, Offset {
			x:      1,
			y:      0,
			width:  input_area.width.saturating_sub(2),
			height: items.len() as u16 + 2,
		});

		if area.y > input_area.y {
			area.y = area.y.saturating_sub(1);
		} else {
			area.y = rect.height.min(area.y + 1);
			area.height = rect.height.saturating_sub(area.y).min(area.height);
		}

		yazi_plugin::elements::Clear::default().render(area, buf);
		List::new(items)
			.block(
				Block::bordered().border_type(BorderType::Rounded).border_style(THEME.completion.border),
			)
			.render(area, buf);
	}
}
