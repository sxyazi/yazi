use ratatui::{buffer::Buffer, widgets::Widget};

use crate::Ctx;

pub(crate) struct Preview<'a> {
	cx: &'a Ctx,
}

impl<'a> Preview<'a> {
	#[inline]
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Preview<'_> {
	fn render(self, area: ratatui::layout::Rect, buf: &mut Buffer) {
		let preview = &self.cx.manager.active().preview;
		let Some(lock) = &preview.lock else {
			return;
		};

		if (lock.window.rows, lock.window.cols) != (area.height, area.width) {
			return;
		}

		for w in &lock.data {
			w.clone_render(buf);
		}
	}
}
