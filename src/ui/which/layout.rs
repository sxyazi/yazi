use ratatui::{layout, prelude::{Buffer, Constraint, Direction, Rect}, widgets::Widget};

use super::Side;
use crate::ui::Ctx;

pub struct Which<'a> {
	cx: &'a mut Ctx,
}

impl<'a> Which<'a> {
	pub fn new(cx: &'a mut Ctx) -> Self { Self { cx } }
}

impl Widget for Which<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let chunks = layout::Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(area);

		let chunks = layout::Layout::default()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(chunks[1]);

		let which = &self.cx.which;
		let cands: (Vec<_>, Vec<_>) = which.cands.iter().enumerate().partition(|(i, _)| i % 2 == 0);

		Side::new(which.times, cands.0).render(chunks[0], buf);
		Side::new(which.times, cands.1).render(chunks[1], buf);
	}
}
