use std::sync::atomic::{AtomicBool, Ordering};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use yazi_adapter::ADAPTOR;

pub static COLLISION: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Default)]
pub struct Clear;

impl Widget for Clear {
	fn render(self, area: Rect, buf: &mut Buffer)
	where
		Self: Sized,
	{
		ratatui::widgets::Clear.render(area, buf);

		let Some(r) = ADAPTOR.get().shown_load().and_then(|r| overlap(area, r)) else {
			return;
		};

		ADAPTOR.get().image_erase(r).ok();
		COLLISION.store(true, Ordering::Relaxed);
		for y in r.top()..r.bottom() {
			for x in r.left()..r.right() {
				buf[(x, y)].set_skip(true);
			}
		}
	}
}

const fn is_overlapping(a: Rect, b: Rect) -> bool {
	a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn overlap(a: Rect, b: Rect) -> Option<Rect> {
	if !is_overlapping(a, b) {
		return None;
	}

	let x = a.x.max(b.x);
	let y = a.y.max(b.y);
	let width = (a.x + a.width).min(b.x + b.width) - x;
	let height = (a.y + a.height).min(b.y + b.height) - y;
	Some(Rect { x, y, width, height })
}
