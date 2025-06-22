use ratatui::{buffer::Buffer, layout::{Margin, Rect}, widgets::{ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget}};
use yazi_config::THEME;
use yazi_core::Scrollable;

use crate::Ctx;

pub(crate) struct List<'a> {
	cx: &'a Ctx,
}

impl<'a> List<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for List<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let pick = &self.cx.pick;

		// Vertical scrollbar
		if pick.len() > pick.limit() {
			Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
				area,
				buf,
				&mut ScrollbarState::new(pick.len()).position(pick.cursor),
			);
		}

		// List content
		let inner = area.inner(Margin::new(1, 0));
		let items = pick.window().map(|(i, v)| {
			if i == pick.cursor {
				ListItem::new(format!(" {v}")).style(THEME.pick.active)
			} else {
				ListItem::new(format!("  {v}")).style(THEME.pick.inactive)
			}
		});
		Widget::render(ratatui::widgets::List::new(items), inner, buf);
	}
}
