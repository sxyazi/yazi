use std::ops::Range;

use ratatui::{layout::Rect, text::Line, widgets::Widget};
use yazi_config::THEME;

use super::Input;

impl Widget for &Input {
	fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		yazi_plugin::elements::Clear::default().render(area, buf);

		Line::styled(self.display(), THEME.input.value).render(area, buf);

		if let Some(Range { start, end }) = self.selected() {
			let s = start.min(area.width);
			buf.set_style(
				Rect {
					x:      area.x + s,
					y:      area.y,
					width:  (end - start).min(area.width - s),
					height: area.height.min(1),
				},
				THEME.input.selected,
			);
		}
	}
}
