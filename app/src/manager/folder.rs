use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::error;

pub(super) enum Folder {
	Parent  = 0,
	Current = 1,
	Preview = 2,
}

impl Widget for Folder {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let folder = plugin::Folder { kind: self as u8 };
		if let Err(e) = folder.render(area, buf) {
			error!("{:?}", e);
		}
	}
}
