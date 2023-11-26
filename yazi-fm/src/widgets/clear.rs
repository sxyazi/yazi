use std::sync::atomic::Ordering;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use yazi_adaptor::ADAPTOR;
use yazi_config::LAYOUT;
use yazi_shared::COLLISION;

pub(crate) struct Clear;

#[inline]
const fn is_overlapping(a: &Rect, b: &Rect) -> bool {
	a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn overlap(a: &Rect, b: &Rect) -> Option<Rect> {
	if !is_overlapping(a, b) {
		return None;
	}

	let x = a.x.max(b.x);
	let y = a.y.max(b.y);
	let width = (a.x + a.width).min(b.x + b.width) - x;
	let height = (a.y + a.height).min(b.y + b.height) - y;
	Some(Rect { x, y, width, height })
}

impl Widget for Clear {
	fn render(self, area: Rect, buf: &mut Buffer) {
		ratatui::widgets::Clear.render(area, buf);

		let Some(r) = overlap(&area, &LAYOUT.load().preview) else {
			return;
		};

		ADAPTOR.image_hide(r).ok();
		COLLISION.store(true, Ordering::Relaxed);
		for x in r.left()..r.right() {
			for y in r.top()..r.bottom() {
				buf.get_mut(x, y).set_skip(true);
			}
		}
	}
}
