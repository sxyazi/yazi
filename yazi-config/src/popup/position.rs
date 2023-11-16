use crossterm::terminal::WindowSize;
use ratatui::layout::Rect;
use yazi_shared::Term;

use super::{Offset, Origin};

#[derive(Clone, Copy, Default)]
pub struct Position {
	pub origin: Origin,
	pub offset: Offset,
}

impl Position {
	#[inline]
	pub fn new(origin: Origin, offset: Offset) -> Self { Self { origin, offset } }

	pub fn rect(&self) -> Rect {
		let Offset { x, y, width, height } = self.offset;
		let WindowSize { columns, rows, .. } = Term::size();

		let max_x = columns.saturating_sub(width);
		let max_y = rows.saturating_sub(height);

		let (x, y) = match self.origin {
			// Top
			Origin::TopLeft => (x.clamp(0, max_x as i16) as u16, y.clamp(0, max_y as i16) as u16),
			Origin::TopCenter => (
				(columns / 2).saturating_sub(width / 2).saturating_add_signed(x).clamp(0, max_x),
				y.clamp(0, max_y as i16) as u16,
			),
			Origin::TopRight => {
				(max_x.saturating_add_signed(x).clamp(0, max_x), y.clamp(0, max_y as i16) as u16)
			}

			// Bottom
			Origin::BottomLeft => {
				(x.clamp(0, max_x as i16) as u16, max_y.saturating_add_signed(y).clamp(0, max_y))
			}
			Origin::BottomCenter => (
				(columns / 2).saturating_sub(width / 2).saturating_add_signed(x).clamp(0, max_x),
				max_y.saturating_add_signed(y).clamp(0, max_y),
			),
			Origin::BottomRight => (
				max_x.saturating_add_signed(x).clamp(0, max_x),
				max_y.saturating_add_signed(y).clamp(0, max_y),
			),

			// Special
			// Origin::Hovered => {
			// 	return Origin::rect(&if let Some(r) =
			// 		self.manager.hovered().and_then(|h| self.manager.current().rect_current(&h.url))
			// 	{
			// 		Origin::Sticky(r)
			// 	} else {
			// 		Origin::TopCenter(rect_shim)
			// 	});
			// }
			Origin::Center => (
				(columns / 2).saturating_sub(width / 2).saturating_add_signed(x).clamp(0, max_x),
				(max_y / 2).saturating_sub(height / 2).saturating_add_signed(y).clamp(0, max_y),
			),
			Origin::Hovered => unreachable!(),
		};

		Rect {
			x,
			y,
			width: width.min(columns.saturating_sub(x)),
			height: height.min(rows.saturating_sub(y)),
		}
	}

	pub fn sticky(base: Rect, offset: Offset) -> Rect {
		let Offset { x, y, width, height } = offset;
		let WindowSize { columns, rows, .. } = Term::size();

		let above =
			base.y.saturating_add(base.height).saturating_add(height).saturating_add_signed(y) > rows;

		let x = base.x.saturating_add_signed(x).clamp(0, columns.saturating_sub(width));
		let y = if above {
			base.y.saturating_sub(height.saturating_sub(y.unsigned_abs()))
		} else {
			base.y.saturating_add(base.height).saturating_add_signed(y)
		};

		Rect {
			x,
			y,
			width: width.min(columns.saturating_sub(x)),
			height: height.min(rows.saturating_sub(y)),
		}
	}
}
