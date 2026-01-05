use ratatui::{buffer::Buffer, layout::Rect, style::Color};

/// Apply background color to a pane, skipping borders on all sides
#[inline]
pub fn apply_pane_bg_with_borders(buf: &mut Buffer, pane: Rect, bg_color: Color) {
	let start_y = pane.top() + 1;
	let end_y = if pane.bottom() > 0 { pane.bottom() - 1 } else { pane.bottom() };
	let start_x = pane.left() + 1;
	let end_x = if pane.right() > 0 { pane.right() - 1 } else { pane.right() };

	if start_y < end_y && start_x < end_x {
		for y in start_y..end_y {
			for x in start_x..end_x {
				buf[(x, y)].set_bg(bg_color);
			}
		}
	}
}

/// Apply background color to a pane, skipping only top/bottom borders
#[inline]
pub fn apply_pane_bg_no_vertical_borders(buf: &mut Buffer, pane: Rect, bg_color: Color) {
	let start_y = pane.top() + 1;
	let end_y = if pane.bottom() > 0 { pane.bottom() - 1 } else { pane.bottom() };

	if start_y < end_y {
		for y in start_y..end_y {
			for x in pane.left()..pane.right() {
				buf[(x, y)].set_bg(bg_color);
			}
		}
	}
}

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
