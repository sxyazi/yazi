use core::Ctx;

use ratatui::{buffer::Buffer, prelude::Rect, widgets::Widget};
use tracing::info;

pub(crate) struct Layout<'a> {
	cx: &'a Ctx,
}

impl<'a> Layout<'a> {
	pub(crate) fn new(cx: &'a Ctx) -> Self { Self { cx } }
}

impl<'a> Widget for Layout<'a> {
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
