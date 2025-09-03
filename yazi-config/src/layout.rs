use ratatui::layout::Rect;

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Layout {
	pub current:  Rect,
	pub preview:  Rect,
	pub progress: Rect,
}

impl Layout {
	pub const fn default() -> Self {
		Self { current: Rect::ZERO, preview: Rect::ZERO, progress: Rect::ZERO }
	}

	pub const fn folder_limit(self) -> usize { self.current.height as _ }
}
