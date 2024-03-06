use std::sync::atomic::Ordering;

use ratatui::{backend::{Backend, CrosstermBackend}, CompletedFrame};
use yazi_plugin::elements::COLLISION;

use crate::{app::App, lives::Lives, root::Root};

impl App {
	pub(crate) fn render(&mut self) {
		let Some(term) = &mut self.term else {
			return;
		};

		let collision = COLLISION.swap(false, Ordering::Relaxed);
		let frame = term
			.draw(|f| {
				_ = Lives::scope(&self.cx, |_| Ok(f.render_widget(Root::new(&self.cx), f.size())));

				if let Some((x, y)) = self.cx.cursor() {
					f.set_cursor(x, y);
				}
			})
			.unwrap();

		if COLLISION.load(Ordering::Relaxed) {
			Self::patch(frame, self.cx.cursor());
		}
		if !self.cx.notify.messages.is_empty() {
			self.render_notify();
		}

		// Reload preview if collision is resolved
		if collision && !COLLISION.load(Ordering::Relaxed) {
			self.cx.manager.peek(true);
		}
	}

	pub(crate) fn render_notify(&mut self) {
		let Some(term) = &mut self.term else {
			return;
		};

		if !term.can_partial() {
			return self.render();
		}

		let frame = term
			.draw_partial(|f| {
				f.render_widget(crate::notify::Layout::new(&self.cx), f.size());

				if let Some((x, y)) = self.cx.cursor() {
					f.set_cursor(x, y);
				}
			})
			.unwrap();

		if COLLISION.load(Ordering::Relaxed) {
			Self::patch(frame, self.cx.cursor());
		}
	}

	#[inline]
	fn patch(frame: CompletedFrame, cursor: Option<(u16, u16)>) {
		let mut patches = vec![];
		for y in frame.area.top()..frame.area.bottom() {
			for x in frame.area.left()..frame.area.right() {
				let cell = frame.buffer.get(x, y);
				if cell.skip {
					patches.push((x, y, cell));
				}
			}
		}

		let mut backend = CrosstermBackend::new(std::io::stdout().lock());
		backend.draw(patches.into_iter()).ok();
		if let Some((x, y)) = cursor {
			backend.show_cursor().ok();
			backend.set_cursor(x, y).ok();
		}
		backend.flush().ok();
	}
}
