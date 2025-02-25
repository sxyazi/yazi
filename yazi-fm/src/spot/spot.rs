use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::Ctx;

pub(crate) struct Spot<'a> {
	cx: &'a Ctx,
}

impl<'a> Spot<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl Widget for Spot<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let Some(lock) = &self.cx.active().spot.lock else {
			return;
		};

		for w in &lock.data {
			w.clone().render(buf, |p| self.cx.mgr.area(p));
		}
	}
}
