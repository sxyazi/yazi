use ratatui::layout::Rect;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Layout {
	pub current: Rect,
	pub preview: Rect,
}
