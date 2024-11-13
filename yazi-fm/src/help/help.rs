use ratatui::{buffer::Buffer, layout::{self, Constraint, Rect}, text::Line, widgets::Widget};
use yazi_config::{KEYMAP, THEME};

use super::Bindings;
use crate::Ctx;

pub(crate) struct Help<'a> {
	cx: &'a Ctx,
}

impl<'a> Help<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }

	fn tips() -> String {
		match KEYMAP.help.iter().find(|&c| c.run.iter().any(|c| c.name == "filter")) {
			Some(c) => format!(" (Press `{}` to filter)", c.on()),
			None => String::new(),
		}
	}
}

impl Widget for Help<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let help = &self.cx.help;
		yazi_plugin::elements::Clear::default().render(area, buf);

		let chunks = layout::Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);
		Line::styled(
			help.keyword().unwrap_or_else(|| format!("{}.help{}", help.layer, Self::tips())),
			THEME.help.footer,
		)
		.render(chunks[1], buf);

		Bindings::new(self.cx).render(chunks[0], buf);
	}
}
