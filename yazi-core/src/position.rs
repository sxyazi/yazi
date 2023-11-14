use ratatui::prelude::Rect;

#[derive(Debug, Clone, Copy)]
pub struct RectShim {
	pub x_offset: i16,
	pub y_offset: i16,
	pub width:    u16,
	pub height:   u16,
}

impl Default for RectShim {
	fn default() -> Self { Self { x_offset: 0, y_offset: 2, width: 50, height: 3 } }
}

#[derive(Debug, Clone)]
pub enum Position {
	TopLeft(RectShim),
	TopRight(RectShim),
	TopCenter(RectShim),
	Center(RectShim),
	BottomCenter(RectShim),
	BottomLeft(RectShim),
	BottomRight(RectShim),
	Hovered(RectShim),
	Sticky(RectShim, Rect),
}

impl Default for Position {
	fn default() -> Self { Self::TopCenter(RectShim::default()) }
}

impl Position {
	#[inline]
	pub fn rect(&self) -> &RectShim {
		match self {
			Position::TopLeft(rect) => rect,
			Position::TopRight(rect) => rect,
			Position::TopCenter(rect) => rect,
			Position::Center(rect) => rect,
			Position::BottomCenter(rect) => rect,
			Position::BottomLeft(rect) => rect,
			Position::BottomRight(rect) => rect,
			Position::Hovered(rect) => rect,
			Position::Sticky(rect, _) => rect,
		}
	}

	#[inline]
	pub fn dimension(&self) -> (u16, u16) { (self.rect().width, self.rect().height) }
}
