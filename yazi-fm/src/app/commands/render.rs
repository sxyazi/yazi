use std::{io::{stderr, BufWriter}, sync::atomic::Ordering};

use crossterm::{execute, queue, terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate}};
use ratatui::{backend::{Backend, CrosstermBackend}, buffer::Buffer, CompletedFrame};
use scopeguard::defer;
use yazi_plugin::elements::COLLISION;

use crate::{app::App, lives::Lives, root::Root};

impl App {
	pub(crate) fn render(&mut self) {
		let Some(term) = &mut self.term else {
			return;
		};

		queue!(stderr(), BeginSynchronizedUpdate).ok();
		defer! { execute!(stderr(), EndSynchronizedUpdate).ok(); }

		let collision = COLLISION.swap(false, Ordering::Relaxed);
		let frame = term
			.draw(|f| {
				_ = Lives::scope(&self.cx, |_| Ok(f.render_widget(Root::new(&self.cx), f.area())));

				if let Some(pos) = self.cx.cursor() {
					f.set_cursor_position(pos);
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
				f.render_widget(crate::notify::Layout::new(&self.cx), f.area());

				if let Some(pos) = self.cx.cursor() {
					f.set_cursor_position(pos);
				}
			})
			.unwrap();

		if COLLISION.load(Ordering::Relaxed) {
			Self::patch(frame, self.cx.cursor());
		}
	}

	#[inline]
	fn patch(frame: CompletedFrame, cursor: Option<(u16, u16)>) {
		let mut new = Buffer::empty(frame.area);
		for y in new.area.top()..new.area.bottom() {
			for x in new.area.left()..new.area.right() {
				let cell = &frame.buffer[(x, y)];
				if cell.skip {
					new[(x, y)] = cell.clone();
				}
				new[(x, y)].set_skip(!cell.skip);
			}
		}

		let patches = frame.buffer.diff(&new);
		let mut backend = CrosstermBackend::new(BufWriter::new(stderr().lock()));
		backend.draw(patches.into_iter()).ok();
		if let Some(pos) = cursor {
			backend.show_cursor().ok();
			backend.set_cursor_position(pos).ok();
		}
		backend.flush().ok();
	}
}
