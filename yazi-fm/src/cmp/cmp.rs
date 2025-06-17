use std::path::MAIN_SEPARATOR_STR;

use ratatui::{buffer::Buffer, layout::Rect, widgets::{Block, BorderType, List, ListItem, Widget}};
use yazi_adapter::Dimension;
use yazi_config::{THEME, popup::{Offset, Position}};

use crate::Ctx;

pub(crate) struct Cmp<'a> {
	cx: &'a Ctx,
}

impl<'a> Cmp<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Cmp<'_> {
	fn render(self, rect: Rect, buf: &mut Buffer) {
		let items: Vec<_> = self
			.cx
			.cmp
			.window()
			.iter()
			.enumerate()
			.map(|(i, x)| {
				let icon = if x.is_dir { &THEME.cmp.icon_folder } else { &THEME.cmp.icon_file };
				let slash = if x.is_dir { MAIN_SEPARATOR_STR } else { "" };

				let mut item = ListItem::new(format!(" {icon} {}{slash}", x.name.display()));
				if i == self.cx.cmp.rel_cursor() {
					item = item.style(THEME.cmp.active);
				} else {
					item = item.style(THEME.cmp.inactive);
				}

				item
			})
			.collect();

		let input_area = self.cx.mgr.area(self.cx.input.position);
		let mut area = Position::sticky(Dimension::available().into(), input_area, Offset {
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
			.block(Block::bordered().border_type(BorderType::Rounded).border_style(THEME.cmp.border))
			.render(area, buf);
	}
}
