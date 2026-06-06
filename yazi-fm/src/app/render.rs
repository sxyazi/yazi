use std::{io::Write, sync::atomic::{AtomicU8, Ordering}, time::Instant};

use anyhow::Result;
use ratatui::layout::Position;
use yazi_actor::{Ctx, lives::Lives};
use yazi_binding::runtime_scope;
use yazi_config::LAYOUT;
use yazi_macro::{act, succ, writef};
use yazi_plugin::LUA;
use yazi_shared::{data::Data, event::NEED_RENDER};
use yazi_term::{CursorStyle, sequence::{BeginSyncUpdate, EndSyncUpdate, MoveTo, SetCursorStyle, ShowCursor}};
use yazi_tty::TTY;
use yazi_widgets::COLLISION;

use crate::{app::App, root::Root};

impl App {
	pub(crate) fn render(&mut self, partial: bool) -> Result<Data> {
		self.last_render = Instant::now();
		NEED_RENDER.store(0, Ordering::Relaxed);
		let Some(term) = &mut self.term else { succ!() };

		if partial {
			return self.render_partially();
		}

		Self::routine(true, None);
		let _guard = scopeguard::guard(self.core.cursor(), |c| Self::routine(false, c));

		let collision = COLLISION.swap(false, Ordering::Relaxed);
		let preview_rect = LAYOUT.get().preview;
		term.draw(|f| {
			_ = Lives::scope(&self.core, || {
				runtime_scope!(LUA, "root", Ok(f.render_widget(Root::new(&self.core), f.area())))
			});
		})?;

		if !self.core.notify.messages.is_empty() {
			self.render_partially()?;
		}

		let cx = &mut Ctx::active(&mut self.core, &mut self.term);
		if collision && !COLLISION.load(Ordering::Relaxed) {
			act!(mgr:peek, cx, true)?; // Reload preview if collision is resolved
		} else if preview_rect != LAYOUT.get().preview {
			act!(mgr:peek, cx)?; // Reload preview if layout changed
		}
		succ!();
	}

	pub(crate) fn render_partially(&mut self) -> Result<Data> {
		let Some(term) = &mut self.term else { succ!() };
		if !term.can_partial() {
			return self.render(false);
		}

		Self::routine(true, None);
		let _guard = scopeguard::guard(self.core.cursor(), |c| Self::routine(false, c));

		term.draw_partial(|f| {
			_ = Lives::scope(&self.core, || {
				runtime_scope!(LUA, "root", {
					f.render_widget(crate::tasks::Progress::new(&self.core), f.area());
					f.render_widget(crate::notify::Notify::new(&self.core), f.area());
					Ok(())
				})
			});
		})?;

		succ!();
	}

	fn routine(push: bool, cursor: Option<(Position, CursorStyle)>) {
		static COUNT: AtomicU8 = AtomicU8::new(0);
		if push && COUNT.fetch_add(1, Ordering::Relaxed) != 0 {
			return;
		} else if !push && COUNT.fetch_sub(1, Ordering::Relaxed) != 1 {
			return;
		}

		if push {
			write!(TTY.writer(), "{BeginSyncUpdate}").ok();
		} else if let Some((Position { x, y }, shape)) = cursor {
			writef!(
				TTY.writer(),
				"{}{}{ShowCursor}{EndSyncUpdate}",
				SetCursorStyle(shape as u8),
				MoveTo(x, y),
			)
			.ok();
		} else {
			writef!(TTY.writer(), "{EndSyncUpdate}").ok();
		};
	}
}
