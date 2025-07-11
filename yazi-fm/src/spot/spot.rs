use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use yazi_core::Core;

pub(crate) struct Spot<'a> {
	core: &'a Core,
}

impl<'a> Spot<'a> {
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Spot<'_> {
	fn render(self, win: Rect, buf: &mut Buffer) {
		let Some(lock) = &self.core.active().spot.lock else {
			return;
		};

		for w in &lock.data {
			let rect = w.area().transform(|p| self.core.mgr.area(p));
			if win.intersects(rect) {
				w.clone().render(rect, buf);
			}
		}
	}
}
