use std::sync::atomic::{AtomicU8, Ordering};

use crossterm::{cursor::{MoveTo, SetCursorStyle, Show}, execute, queue, terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate}};
use ratatui::{CompletedFrame, backend::{Backend, CrosstermBackend}, buffer::Buffer, layout::Position};
use yazi_plugin::elements::COLLISION;
use yazi_shared::event::NEED_RENDER;
use yazi_term::tty::TTY;

use crate::{app::App, lives::Lives, root::Root};

impl App {
	pub(crate) fn render(&mut self) {
		NEED_RENDER.store(false, Ordering::Relaxed);
		let Some(term) = &mut self.term else { return };

		Self::routine(true, None);
		let _guard = scopeguard::guard(self.cx.cursor(), |c| Self::routine(false, c));

		let collision = COLLISION.swap(false, Ordering::Relaxed);
		let frame = term
			.draw(|f| {
				_ = Lives::scope(&self.cx, || Ok(f.render_widget(Root::new(&self.cx), f.area())));
			})
			.unwrap();

		if COLLISION.load(Ordering::Relaxed) {
			Self::patch(frame);
		}
		if !self.cx.notify.messages.is_empty() {
			self.render_partially();
		}

		// Reload preview if collision is resolved
		if collision && !COLLISION.load(Ordering::Relaxed) {
			self.cx.mgr.peek(true);
		}
	}

	pub(crate) fn render_partially(&mut self) {
		let Some(term) = &mut self.term else { return };
		if !term.can_partial() {
			return self.render();
		}

		Self::routine(true, None);
		let _guard = scopeguard::guard(self.cx.cursor(), |c| Self::routine(false, c));

		let frame = term
			.draw_partial(|f| {
				_ = Lives::scope(&self.cx, || {
					f.render_widget(crate::tasks::Progress::new(&self.cx), f.area());
					f.render_widget(crate::notify::Notify::new(&self.cx), f.area());
					Ok(())
				});
			})
			.unwrap();

		if COLLISION.load(Ordering::Relaxed) {
			Self::patch(frame);
		}
	}

	#[inline]
	fn patch(frame: CompletedFrame) {
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
		CrosstermBackend::new(&mut *TTY.lockout()).draw(patches.into_iter()).ok();
	}

	fn routine(push: bool, cursor: Option<(Position, SetCursorStyle)>) {
		static COUNT: AtomicU8 = AtomicU8::new(0);
		if push && COUNT.fetch_add(1, Ordering::Relaxed) != 0 {
			return;
		} else if !push && COUNT.fetch_sub(1, Ordering::Relaxed) != 1 {
			return;
		}

		_ = if push {
			queue!(TTY.writer(), BeginSynchronizedUpdate)
		} else if let Some((Position { x, y }, shape)) = cursor {
			execute!(TTY.writer(), shape, MoveTo(x, y), Show, EndSynchronizedUpdate)
		} else {
			execute!(TTY.writer(), EndSynchronizedUpdate)
		};
	}
}
