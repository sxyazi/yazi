use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tracing::info;

pub(super) struct Folder {
	kind: FolderKind,
}

pub(super) enum FolderKind {
	Parent  = 0,
	Current = 1,
	Preview = 2,
}

impl Folder {
	pub(super) fn new(kind: FolderKind) -> Self { Self { kind } }
}

impl Widget for Folder {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let x = plugin::Folder { kind: self.kind as u8 }.render(area, buf);
		if x.is_err() {
			info!("{:?}", x);
		}
	}
}
