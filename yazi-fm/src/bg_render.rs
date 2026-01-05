use ratatui::{buffer::Buffer, layout::Rect, style::Color};

/// Apply background color to entire area, excluding header and status bar
#[inline]
pub fn apply_overall_bg(buf: &mut Buffer, area: Rect, bg_color: Color) {
	// Skip the header (top row) and status bar (bottom row)
	let start_y = area.top() + 1;
	let end_y = if area.bottom() > 0 { area.bottom() - 1 } else { area.bottom() };

	if start_y < end_y {
		for y in start_y..end_y {
			for x in area.left()..area.right() {
				buf[(x, y)].set_bg(bg_color);
			}
		}
	}
}
