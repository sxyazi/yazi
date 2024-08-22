use crossterm::terminal::WindowSize;
use ratatui::layout::Rect;

use super::{Offset, Origin};

#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
	pub origin: Origin,
	pub offset: Offset,
}

impl Position {
	#[inline]
	pub fn new(origin: Origin, offset: Offset) -> Self { Self { origin, offset } }

	pub fn rect(&self, WindowSize { columns, rows, .. }: WindowSize) -> Rect {
		use Origin::*;
		let Offset { x, y, width, height } = self.offset;

		let max_x = columns.saturating_sub(width);
		let new_x = match self.origin {
			TopLeft | BottomLeft => x.clamp(0, max_x as i16) as u16,
			TopCenter | BottomCenter | Center => {
				(columns / 2).saturating_sub(width / 2).saturating_add_signed(x).clamp(0, max_x)
			}
			TopRight | BottomRight => max_x.saturating_add_signed(x).clamp(0, max_x),
			Hovered => unreachable!(),
		};

		let max_y = rows.saturating_sub(height);
		let new_y = match self.origin {
			TopLeft | TopCenter | TopRight => y.clamp(0, max_y as i16) as u16,
			Center => (rows / 2).saturating_sub(height / 2).saturating_add_signed(y).clamp(0, max_y),
			BottomLeft | BottomCenter | BottomRight => max_y.saturating_add_signed(y).clamp(0, max_y),
			Hovered => unreachable!(),
		};

		Rect {
			x:      new_x,
			y:      new_y,
			width:  width.min(columns.saturating_sub(new_x)),
			height: height.min(rows.saturating_sub(new_y)),
		}
	}

	pub fn sticky(WindowSize { columns, rows, .. }: WindowSize, base: Rect, offset: Offset) -> Rect {
		let Offset { x, y, width, height } = offset;

		let above =
			base.y.saturating_add(base.height).saturating_add(height).saturating_add_signed(y) > rows;

		let new_x = base.x.saturating_add_signed(x).clamp(0, columns.saturating_sub(width));
		let new_y = if above {
			base.y.saturating_sub(height.saturating_sub(y.unsigned_abs()))
		} else {
			base.y.saturating_add(base.height).saturating_add_signed(y)
		};

		Rect {
			x:      new_x,
			y:      new_y,
			width:  width.min(columns.saturating_sub(new_x)),
			height: height.min(rows.saturating_sub(new_y)),
		}
	}
}
