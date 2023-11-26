use ratatui::{buffer::Buffer, layout::{self, Rect}, prelude::{Constraint, Direction}, widgets::{Paragraph, Widget}};
use yazi_config::THEME;

use super::Bindings;
use crate::{Ctx, widgets};

pub(crate) struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Layout<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks = layout::Layout::new()
			.direction(Direction::Vertical)
			.constraints([Constraint::Min(0), Constraint::Length(1)])
			.split(area);

		widgets::Clear.render(area, buf);

		let help = &self.cx.help;
		Paragraph::new(help.keyword().unwrap_or_else(|| format!("{}.help", help.layer)))
			.style(THEME.help.footer.into())
			.render(chunks[1], buf);

		Bindings::new(self.cx).render(chunks[0], buf);
	}
}
