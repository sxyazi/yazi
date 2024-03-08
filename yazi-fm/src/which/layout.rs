use ratatui::{buffer::Buffer, layout, layout::{Constraint, Rect}, widgets::{Block, Widget}};
use yazi_config::THEME;

use super::Cand;
use crate::Ctx;

const PADDING_X: u16 = 1;
const PADDING_Y: u16 = 1;

pub(crate) struct Which<'a> {
	cx: &'a Ctx,
}

impl<'a> Which<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Which<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let which = &self.cx.which;
		if which.silent {
			return;
		}

		let cols = THEME.which.cols as usize;
		let height = area.height.min(which.cands.len().div_ceil(cols) as u16 + PADDING_Y * 2);
		let area = Rect {
			x: PADDING_X.min(area.width),
			y: area.height.saturating_sub(height + PADDING_Y * 2),
			width: area.width.saturating_sub(PADDING_X * 2),
			height,
		};

		// Don't render if there's no space
		if area.height <= PADDING_Y * 2 {
			return;
		}

		let chunks = {
			use Constraint::*;
			layout::Layout::horizontal(match cols {
				1 => &[Ratio(1, 1)][..],
				2 => &[Ratio(1, 2), Ratio(1, 2)],
				_ => &[Ratio(1, 3), Ratio(1, 3), Ratio(1, 3)],
			})
			.split(area)
		};

		yazi_plugin::elements::Clear::default().render(area, buf);
		Block::new().style(THEME.which.mask).render(area, buf);

		for y in 0..area.height {
			for (x, chunk) in chunks.iter().enumerate() {
				let Some(cand) = which.cands.get(y as usize * cols + x) else {
					break;
				};

				Cand::new(cand, which.times).render(Rect { y: chunk.y + y + 1, height: 1, ..*chunk }, buf);
			}
		}
	}
}
