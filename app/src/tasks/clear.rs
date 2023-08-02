use ratatui::{buffer::Buffer, layout::Rect, widgets::{self, Widget}};

pub(super) struct Clear;

impl Widget for Clear {
	fn render(self, mut area: Rect, buf: &mut Buffer) {
		if area.x > 0 {
			area.x -= 1;
			area.width += 2;
		}

		if area.y > 0 {
			area.y -= 1;
			area.height += 2;
		}

		widgets::Clear.render(area, buf)
	}
}
