use ratatui::prelude::Rect;

#[derive(Default)]
pub enum Position {
	#[default]
	None,
	Top(Rect),
	Hovered(Rect),
}

impl Position {
	#[inline]
	pub fn rect(&self) -> Option<Rect> {
		match self {
			Position::None => None,
			Position::Top(rect) => Some(*rect),
			Position::Hovered(rect) => Some(*rect),
		}
	}

	#[inline]
	pub fn dimension(&self) -> Option<(u16, u16)> { self.rect().map(|r| (r.width, r.height)) }
}
