use ratatui::{buffer::Buffer, layout::{self, Constraint, Direction, Rect}, widgets::{Paragraph, Widget}};
use yazi_config::THEME;

use super::Bindings;
use crate::{widgets, Ctx};

pub(crate) struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Layout<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks =
			layout::Layout::new(Direction::Vertical, [Constraint::Min(0), Constraint::Length(1)])
				.split(area);

		widgets::Clear.render(area, buf);

		let help = &self.cx.help;
		Paragraph::new(help.keyword().unwrap_or_else(|| format!("{}.help", help.layer.to_string())))
			.style(THEME.help.footer.into())
			.render(chunks[1], buf);

		Bindings::new(self.cx).render(chunks[0], buf);
	}
}
