use std::path::MAIN_SEPARATOR_STR;

use ratatui_core::{buffer::Buffer, layout::Rect, widgets::Widget};
use ratatui_widgets::{block::Block, borders::BorderType, list::{List, ListItem}};
use yazi_binding::position::{Offset, Position};
use yazi_config::THEME;
use yazi_core::Core;
use yazi_shared::strand::StrandLike;
use yazi_term::TERM;

pub(crate) struct Cmp<'a> {
	core: &'a Core,
}

impl<'a> Cmp<'a> {
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Cmp<'_> {
	fn render(self, rect: Rect, buf: &mut Buffer) {
		let items: Vec<_> = self
			.core
			.cmp
			.window()
			.iter()
			.enumerate()
			.map(|(i, x)| {
				let icon = if x.is_dir { &THEME.cmp.icon_folder } else { &THEME.cmp.icon_file };
				let slash = if x.is_dir { MAIN_SEPARATOR_STR } else { "" };

				let mut item = ListItem::new(format!(" {icon} {}{slash}", x.name.display()));
				if i == self.core.cmp.rel_cursor() {
					item = item.style(THEME.cmp.active.get());
				} else {
					item = item.style(THEME.cmp.inactive.get());
				}

				item
			})
			.collect();

		let input_area = self.core.mgr.area(self.core.input.main.position);
		let mut area = Position::hovered(Offset {
			x:      1,
			y:      0,
			width:  input_area.width.saturating_sub(2),
			height: items.len() as u16 + 2,
		})
		.sticky(input_area, TERM.dimension().area());

		if area.y > input_area.y {
			area.y = area.y.saturating_sub(1);
		} else {
			area.y = rect.height.min(area.y + 1);
			area.height = rect.height.saturating_sub(area.y).min(area.height);
		}

		yazi_widgets::clear::Clear::default().render(area, buf);
		List::new(items)
			.block(
				Block::bordered().border_type(BorderType::Rounded).border_style(THEME.cmp.border.get()),
			)
			.render(area, buf);
	}
}
