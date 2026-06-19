use std::ops::{Deref, DerefMut};

use ratatui_core::layout::Rect;
use ratatui_widgets::block::Padding;
use yazi_shim::ratatui::Padable;

use super::{Offset, Origin};

#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
	pub origin:  Origin,
	pub offset:  Offset,
	pub padding: Padding,
}

impl Deref for Position {
	type Target = Offset;

	fn deref(&self) -> &Self::Target { &self.offset }
}

impl DerefMut for Position {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.offset }
}

impl From<Rect> for Position {
	fn from(value: Rect) -> Self { Self::new(Origin::TopLeft, value.into()) }
}

impl Position {
	pub const fn new(origin: Origin, offset: Offset) -> Self {
		Self { origin, offset, padding: Padding::ZERO }
	}

	pub const fn hovered(offset: Offset) -> Self {
		Self { origin: Origin::Hovered, offset, padding: Padding::ZERO }
	}

	pub fn with_height(mut self, height: u16) -> Self {
		self.height = height;
		self
	}

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
		.padding(self.padding)
	}

	pub fn sticky(self, base: Rect, (cols, rows): (u16, u16)) -> Rect {
		let Offset { x, y, width, height } = self.offset;

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
		.padding(self.padding)
	}

	pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
		let p = padding.into();
		self.padding.left = self.padding.left.saturating_add(p.left);
		self.padding.right = self.padding.right.saturating_add(p.right);
		self.padding.top = self.padding.top.saturating_add(p.top);
		self.padding.bottom = self.padding.bottom.saturating_add(p.bottom);
		self
	}
}
