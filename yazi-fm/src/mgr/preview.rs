use ratatui::{buffer::Buffer, widgets::Widget};
use yazi_config::LAYOUT;
use yazi_core::Core;

pub(crate) struct Preview<'a> {
	core: &'a Core,
}

impl<'a> Preview<'a> {
	#[inline]
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }
}

impl Widget for Preview<'_> {
	fn render(self, win: ratatui::layout::Rect, buf: &mut Buffer) {
		let Some(lock) = &self.core.active().preview.lock else {
			return;
		};

		if *lock.area != LAYOUT.get().preview {
			return;
		}

		for w in &lock.data {
			let rect = w.area().transform(|p| self.core.mgr.area(p));
			if win.intersects(rect) {
				w.clone().render(rect, buf);
			}
		}
	}
}
