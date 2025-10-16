use ratatui::{buffer::Buffer, layout::{Margin, Rect}, widgets::{ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget}};
use yazi_config::{THEME, YAZI};
use yazi_core::Core;
use yazi_widgets::Scrollable;

pub(crate) struct List<'a> {
	core: &'a Core,
}

impl<'a> List<'a> {
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for List<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let pick = &self.core.pick;

		// Vertical scrollbar
		if pick.total() > pick.limit() {
			Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
				area,
				buf,
				&mut ScrollbarState::new(pick.total()).position(pick.cursor),
			);
		}

		// List content
		let inner = area.inner(Margin::new(1, 0));
		let items = pick.window().map(|(i, v)| {
			let (prefix, style) =
				if i == pick.cursor { ("", THEME.pick.active) } else { (" ", THEME.pick.inactive) };

			let index_str = if !YAZI.pick.line_numbers {
				"".to_string()
			} else {
				if i < 9 { format!("{:>2}", i + 1) } else { "  ".to_string() }
			};

			ListItem::new(format!("{prefix}{index_str} {v}")).style(style)
		});
		Widget::render(ratatui::widgets::List::new(items), inner, buf);
	}
}
