use ratatui::{prelude::{Buffer, Rect}, widgets::Widget};
use tracing::info;

pub(crate) struct Layout;

impl Widget for Layout {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let x = plugin::Header.render(area, buf);
		if x.is_err() {
			info!("{:?}", x);
		}
	}
}
