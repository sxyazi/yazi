use ratatui::{layout, prelude::{Buffer, Constraint, Direction, Rect}, widgets::{Block, Widget}};
use yazi_config::THEME;
use yazi_core::Ctx;

use super::Side;
use crate::widgets;

pub(crate) struct Which<'a> {
	cx: &'a Ctx,
}

impl<'a> Which<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Which<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let which = &self.cx.which;
		let mut cands: (Vec<_>, Vec<_>, Vec<_>) = Default::default();
		for (i, c) in which.cands.iter().enumerate() {
			match i % 3 {
				0 => cands.0.push(c),
				1 => cands.1.push(c),
				2 => cands.2.push(c),
				_ => unreachable!(),
			}
		}

		let height = cands.0.len() as u16 + 2;
		let area = Rect {
			x: 1,
			y: area.height.saturating_sub(height + 2),
			width: area.width.saturating_sub(2),
			height,
		};

		let chunks = layout::Layout::new()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
			.split(area);

		widgets::Clear.render(area, buf);
		Block::new().style(THEME.which.mask.into()).render(area, buf);
		Side::new(which.times, cands.0).render(chunks[0], buf);
		Side::new(which.times, cands.1).render(chunks[1], buf);
		Side::new(which.times, cands.2).render(chunks[2], buf);
	}
}
