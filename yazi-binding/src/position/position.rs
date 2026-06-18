use std::ops::{Deref, DerefMut};

use ratatui_core::layout::Rect;
use ratatui_widgets::block::Padding;

use super::{Offset, Origin};

#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
	pub origin: Origin,
	pub offset: Offset,
}

impl Deref for Position {
	type Target = Offset;

	fn deref(&self) -> &Self::Target { &self.offset }
}

impl DerefMut for Position {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.offset }
}

impl Position {
	pub const fn new(origin: Origin, offset: Offset) -> Self { Self { origin, offset } }

	pub fn rect(&self, (cols, rows): (u16, u16)) -> Rect {
		use Origin::*;
		let Offset { x, y, width, height } = self.offset;

		let max_x = cols.saturating_sub(width);
		let new_x = match self.origin {
			TopLeft | BottomLeft => x.clamp(0, max_x as i16) as u16,
			TopCenter | BottomCenter | Center => {
				(cols / 2).saturating_sub(width / 2).saturating_add_signed(x).clamp(0, max_x)
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
			width:  width.min(cols.saturating_sub(new_x)),
			height: height.min(rows.saturating_sub(new_y)),
		}
	}

	pub fn sticky((cols, rows): (u16, u16), base: Rect, offset: Offset) -> Rect {
		let Offset { x, y, width, height } = offset;

		let above =
			base.y.saturating_add(base.height).saturating_add(height).saturating_add_signed(y) > rows;

		let new_x = base.x.saturating_add_signed(x).clamp(0, cols.saturating_sub(width));
		let new_y = if above {
			base.y.saturating_sub(height.saturating_sub(y.unsigned_abs()))
		} else {
			base.y.saturating_add(base.height).saturating_add_signed(y)
		};

		Rect {
			x:      new_x,
			y:      new_y,
			width:  width.min(cols.saturating_sub(new_x)),
			height: height.min(rows.saturating_sub(new_y)),
		}
	}

	pub fn padding(mut self, padding: Padding) -> Self {
		use Origin::*;
		let h_reduction = padding.left + padding.right;
		let v_reduction = padding.top + padding.bottom;

		self.x = self.x.saturating_add_unsigned(padding.left);
		self.y = self.y.saturating_add_unsigned(padding.top);

		self.width = self.width.saturating_sub(h_reduction);
		self.height = self.height.saturating_sub(v_reduction);

		self.x = self.x.saturating_sub_unsigned(match self.origin {
			TopCenter | BottomCenter | Center => h_reduction / 2,
			TopRight | BottomRight => h_reduction,
			_ => 0,
		});

		self.y = self.y.saturating_sub_unsigned(match self.origin {
			BottomLeft | BottomCenter | BottomRight => v_reduction,
			Center => v_reduction / 2,
			_ => 0,
		});

		self
	}
}
