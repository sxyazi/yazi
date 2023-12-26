use ratatui::layout::Rect;

#[derive(Default)]
pub struct Layout {
	pub header: Rect,

	pub parent:  Rect,
	pub current: Rect,
	pub preview: Rect,

	pub status: Rect,
}
