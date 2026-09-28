use ratatui_core::{buffer::Buffer, layout::{Constraint, Layout, Rect}, widgets::Widget};
use ratatui_widgets::{block::Block, borders::BorderType};
use yazi_config::{THEME, YAZI};
use yazi_core::Core;
use yazi_widgets::clear::Clear;

use super::Cand;

const PADDING_X: u16 = 1;
const PADDING_Y: u16 = 1;

pub(crate) struct Which<'a> {
	core: &'a Core,
}

impl<'a> Which<'a> {
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Which<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let which = &self.core.which;
		if which.silent {
			return;
		}

		let rows = Cand::rows(&which.cands, which.times, YAZI.which.fold);
		let cols = THEME.which.cols.get() as usize;
		let height = area.height.min(rows.len().div_ceil(cols) as u16 + PADDING_Y * 2);
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

		Clear::default().render(area, buf);
		let block = Block::bordered()
			.style(THEME.which.mask.get())
			.border_type(BorderType::Rounded)
			.border_style(THEME.which.border.get());
		block.as_ref().render(area, buf);

		let inner = block.inner(area);
		let chunks = {
			use Constraint::*;
			Layout::horizontal(match cols {
				1 => &[Ratio(1, 1)][..],
				2 => &[Ratio(1, 2), Ratio(1, 2)],
				_ => &[Ratio(1, 3), Ratio(1, 3), Ratio(1, 3)],
			})
			.split(inner)
		};

		let mut rows = rows.into_iter();
		'fill: for y in 0..inner.height {
			for chunk in chunks.iter() {
				let Some(row) = rows.next() else {
					break 'fill;
				};

				row.render(Rect { y: chunk.y + y, height: 1, ..*chunk }, buf);
			}
		}
	}
}
