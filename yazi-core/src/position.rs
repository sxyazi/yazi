use ratatui::prelude::Rect;

#[derive(Debug, Clone, Copy)]
pub struct Offset {
	pub x_offset: i16,
	pub y_offset: i16,
	pub width:    u16,
	pub height:   u16,
}

impl Default for Offset {
	fn default() -> Self { Self { x_offset: 0, y_offset: 2, width: 50, height: 3 } }
}

#[derive(Debug, Clone)]
pub enum Position {
	TopLeft(Offset),
	TopRight(Offset),
	TopCenter(Offset),
	Center(Offset),
	BottomCenter(Offset),
	BottomLeft(Offset),
	BottomRight(Offset),
	Hovered(Offset),
	Sticky(Offset, Rect),
}

impl Default for Position {
	fn default() -> Self { Self::TopCenter(Offset::default()) }
}

impl Position {
	#[inline]
	pub fn offset(&self) -> &Offset {
		match self {
			Position::TopLeft(offset) => offset,
			Position::TopRight(offset) => offset,
			Position::TopCenter(offset) => offset,
			Position::Center(offset) => offset,
			Position::BottomCenter(offset) => offset,
			Position::BottomLeft(offset) => offset,
			Position::BottomRight(offset) => offset,
			Position::Hovered(offset) => offset,
			Position::Sticky(offset, _) => offset,
		}
	}

	#[inline]
	pub fn dimension(&self) -> (u16, u16) { (self.offset().width, self.offset().height) }
}
