use std::ops::Range;

use ratatui_core::{buffer::Buffer, layout::Rect, text::Line, widgets::Widget};

use super::Input;

impl Widget for &Input {
	fn render(self, area: Rect, buf: &mut Buffer) {
		buf.set_style(area, self.styles.normal.unwrap_or_default());

		// Hint
		if self.value().is_empty() {
			Line::styled(&self.hint, self.styles.hint.unwrap_or_default()).render(area, buf);
			return;
		}

		// Input value
		Line::raw(self.display()).render(area, buf);

		// Selection
		if let Some(Range { start, end }) = self.selected() {
			let s = start.min(area.width);
			buf.set_style(
				Rect {
					x:      area.x + s,
					y:      area.y,
					width:  (end - start).min(area.width - s),
					height: area.height.min(1),
				},
				self.styles.selected.unwrap_or_default(),
			);
		}
	}
}
