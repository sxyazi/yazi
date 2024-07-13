use ratatui::layout::Rect;

#[derive(Clone, Copy, Default)]
pub struct Layout {
	pub current: Rect,
	pub preview: Rect,
}
