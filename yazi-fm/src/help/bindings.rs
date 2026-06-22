use ratatui_core::{buffer::Buffer, layout::{self, Constraint, Rect}, widgets::Widget};
use ratatui_widgets::list::{List, ListItem};
use yazi_config::THEME;
use yazi_core::Core;

pub(super) struct Bindings<'a> {
	core: &'a Core,
}

impl<'a> Bindings<'a> {
	pub(super) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Bindings<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let bindings = &self.core.help.window();
		if bindings.is_empty() {
			return;
		}

		// Chord
		let col1: Vec<_> =
			bindings.iter().map(|c| ListItem::new(c.on()).style(THEME.help.chord.get())).collect();

		// Action
		let col2: Vec<_> = bindings
			.iter()
			.map(|c| ListItem::new(c.desc_or_run()).style(THEME.help.action.get()))
			.collect();

		let chunks =
			layout::Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).split(area);

		let cursor = self.core.help.rel_cursor() as u16;
		buf.set_style(
			Rect { x: area.x, y: area.y + cursor, width: area.width, height: 1 },
			THEME.help.hovered.get(),
		);

		List::new(col1).render(chunks[0], buf);
		List::new(col2).render(chunks[1], buf);
	}
}
