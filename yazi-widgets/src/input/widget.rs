use std::ops::Range;

use ratatui_core::{buffer::Buffer, layout::Rect, text::Line, widgets::Widget};

use super::Input;

impl Widget for &Input {
	fn render(self, area: Rect, buf: &mut Buffer)
	where
		Self: Sized,
	{
		let normal = self.styles.normal.unwrap_or_default();
		let selected = self.styles.selected.unwrap_or_default();

		Line::styled(self.display(), normal).render(area, buf);

		if let Some(Range { start, end }) = self.selected() {
			let s = start.min(area.width);
			buf.set_style(
				Rect {
					x:      area.x + s,
					y:      area.y,
					width:  (end - start).min(area.width - s),
					height: area.height.min(1),
				},
				selected,
			);
		}
	}
}
