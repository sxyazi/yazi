use ratatui_core::{buffer::Buffer, layout::{Alignment, Constraint, Layout, Rect}, symbols::merge::MergeStrategy, widgets::Widget};
use ratatui_widgets::{block::{Block, Padding}, borders::BorderType};
use yazi_config::THEME;
use yazi_core::Core;
use yazi_shim::ratatui::Padable;

use super::Bindings;

pub(crate) struct Help<'a> {
	core: &'a Core,
}

impl<'a> Help<'a> {
	pub fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Help<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let help = &self.core.help;
		let area = self.core.mgr.area(help.position);
		let padding = help.padding();

		yazi_widgets::clear::Clear::default().render(area, buf);

		Block::bordered()
			.title(format!("{}.help", help.layer))
			.title_alignment(Alignment::Center)
			.border_type(BorderType::Rounded)
			.border_style(THEME.help.border.get())
			.render(area, buf);

		let chunks =
			Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Fill(1)])
				.split(area.padding(Padding { left: 0, right: 0, ..padding }));

		// Input
		help.input.render(chunks[0].padding(Padding { top: 0, bottom: 0, ..padding }), buf);

		// Divider
		Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(THEME.help.border.get())
			.merge_borders(MergeStrategy::Fuzzy)
			.render(chunks[1], buf);

		// Bindings
		Bindings::new(self.core)
			.render(chunks[2].padding(Padding { top: 0, bottom: 0, ..padding }), buf);
	}
}
