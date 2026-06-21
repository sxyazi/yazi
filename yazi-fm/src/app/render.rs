use std::{sync::atomic::Ordering, time::Instant};

use anyhow::Result;
use yazi_actor::{Ctx, lives::Lives};
use yazi_adapter::ADAPTOR;
use yazi_binding::runtime_scope;
use yazi_config::LAYOUT;
use yazi_macro::{act, succ};
use yazi_plugin::LUA;
use yazi_shared::{data::Data, event::NEED_RENDER};

use super::SyncGuard;
use crate::{app::App, root::Root};

impl App {
	pub(crate) fn render(&mut self, partial: bool) -> Result<Data> {
		self.last_render = Instant::now();
		NEED_RENDER.store(0, Ordering::Relaxed);
		let Some(term) = &mut self.term else { succ!() };

		if partial {
			return self.render_partially();
		}

		let guard = SyncGuard::enter();
		let collision = ADAPTOR.collision.replace(false);
		let preview_rect = LAYOUT.get().preview;
		term.draw(|f| {
			_ = Lives::scope(&mut self.core, |core| {
				runtime_scope!(LUA, "root", Ok(f.render_widget(Root::new(core), f.area())))
			});
		})?;

		if !self.core.notify.messages.is_empty() {
			self.render_partially()?;
		}

		let cx = &mut Ctx::active(&mut self.core, &mut self.term);
		if collision && !ADAPTOR.collision.get() {
			act!(mgr:peek, cx, true)?; // Reload preview if collision is resolved
		} else if preview_rect != LAYOUT.get().preview {
			act!(mgr:peek, cx)?; // Reload preview if layout changed
		}

		guard.finish(self.core.cursor());
		succ!();
	}

	pub(crate) fn render_partially(&mut self) -> Result<Data> {
		let Some(term) = &mut self.term else { succ!() };
		if !term.can_partial() {
			return self.render(false);
		}

		let guard = SyncGuard::enter();
		term.draw_partial(|f| {
			_ = Lives::scope(&mut self.core, |core| {
				runtime_scope!(LUA, "root", {
					f.render_widget(crate::tasks::Progress::new(core), f.area());
					f.render_widget(crate::notify::Notify::new(core), f.area());
					Ok(())
				})
			});
		})?;

		guard.finish(self.core.cursor());
		succ!();
	}
}
