use std::sync::atomic::{AtomicU8, Ordering};

use anyhow::Result;
use crossterm::{cursor::{MoveTo, SetCursorStyle, Show}, execute, queue, terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate}};
use ratatui::{CompletedFrame, backend::{Backend, CrosstermBackend}, buffer::Buffer, layout::Position};
use yazi_actor::{Ctx, lives::Lives};
use yazi_binding::elements::COLLISION;
use yazi_macro::{act, succ};
use yazi_parser::VoidOpt;
use yazi_shared::{data::Data, event::NEED_RENDER};
use yazi_term::tty::TTY;

use crate::{app::App, root::Root};

impl App {
	pub(crate) fn render(&mut self, _: VoidOpt) -> Result<Data> {
		NEED_RENDER.store(false, Ordering::Relaxed);
		let Some(term) = &mut self.term else { succ!() };

		Self::routine(true, None);
		let _guard = scopeguard::guard(self.core.cursor(), |c| Self::routine(false, c));

		let collision = COLLISION.swap(false, Ordering::Relaxed);
		let frame = term
			.draw(|f| {
				_ = Lives::scope(&self.core, || Ok(f.render_widget(Root::new(&self.core), f.area())));
			})
			.unwrap();

		if COLLISION.load(Ordering::Relaxed) {
			Self::patch(frame);
		}
		if !self.core.notify.messages.is_empty() {
			act!(render_partially, self)?;
		}

		// Reload preview if collision is resolved
		if collision && !COLLISION.load(Ordering::Relaxed) {
			let cx = &mut Ctx::active(&mut self.core);
			act!(mgr:peek, cx, true)?;
		}
		succ!();
	}

	pub(crate) fn render_partially(&mut self, _: VoidOpt) -> Result<Data> {
		let Some(term) = &mut self.term else { succ!() };
		if !term.can_partial() {
			return act!(render, self);
		}

		Self::routine(true, None);
		let _guard = scopeguard::guard(self.core.cursor(), |c| Self::routine(false, c));

		let frame = term
			.draw_partial(|f| {
				_ = Lives::scope(&self.core, || {
					f.render_widget(crate::tasks::Progress::new(&self.core), f.area());
					f.render_widget(crate::notify::Notify::new(&self.core), f.area());
					Ok(())
				});
			})
			.unwrap();

		if COLLISION.load(Ordering::Relaxed) {
			Self::patch(frame);
		}
		succ!();
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
