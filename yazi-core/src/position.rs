use ratatui::prelude::Rect;

pub enum Position {
	Top(Rect),
	Sticky(Rect, Rect),
	Hovered(Rect),
}

impl Default for Position {
	fn default() -> Self { Self::Top(Rect::default()) }
}

impl Position {
	#[inline]
	pub fn rect(&self) -> Rect {
		match self {
			Position::Top(rect) => *rect,
			Position::Sticky(rect, _) => *rect,
			Position::Hovered(rect) => *rect,
		}
	}

	#[inline]
	pub fn dimension(&self) -> (u16, u16) { (self.rect().width, self.rect().height) }
}
