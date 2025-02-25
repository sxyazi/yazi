use ratatui::{buffer::Buffer, widgets::Widget};
use yazi_config::LAYOUT;

use crate::Ctx;

pub(crate) struct Preview<'a> {
	cx: &'a Ctx,
}

impl<'a> Preview<'a> {
	#[inline]
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Preview<'_> {
	fn render(self, _: ratatui::layout::Rect, buf: &mut Buffer) {
		let Some(lock) = &self.cx.active().preview.lock else {
			return;
		};

		if *lock.area != LAYOUT.get().preview {
			return;
		}

		for w in &lock.data {
			w.clone().render(buf, |p| self.cx.mgr.area(p));
		}
	}
}
