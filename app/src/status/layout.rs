use ratatui::{buffer::Buffer, prelude::Rect, widgets::Widget};
use tracing::info;

pub(crate) struct Layout;

impl Widget for Layout {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let x = plugin::Status::render(area);
		if x.is_err() {
			info!("{:?}", x);
			return;
		}

		for x in x.unwrap() {
			x.render(buf);
		}
	}
}
