use std::{io, sync::atomic::Ordering, time::Instant};

use anyhow::Result;
use ratatui_core::{buffer::{Buffer, CellDiffOption}, layout::Position};
use yazi_actor::{act, lives::Lives};
use yazi_adapter::ADAPTOR;
use yazi_binding::runtime_scope;
use yazi_config::LAYOUT;
use yazi_core::{Core, Ctx};
use yazi_macro::succ;
use yazi_plugin::LUA;
use yazi_shared::{data::Data, event::NEED_RENDER};
use yazi_tui::Raterm;

use super::SyncGuard;
use crate::{app::App, root::Root};

impl App {
	pub(crate) fn render(&mut self) -> Result<Data> {
		self.last_render = Instant::now();
		NEED_RENDER.store(0, Ordering::Relaxed);

		if self.need_render == /* partial */ 1 {
			return self.render_partially();
		}
		let Some(term) = self.core.term.take() else {
			succ!();
		};

		let guard = SyncGuard::enter();
		let collision = ADAPTOR.collision.replace(false);
		let preview_rect = LAYOUT.get().preview;
		Self::scope(&mut self.core, term, |core, term| {
			term.draw(|f| {
				f.render_widget(Root::new(core), f.area());
				if self.need_render == /* force */ 3 {
					Self::render_forcibly(f.buffer_mut());
				}
			})
		})?;

		if !self.core.notify.messages.is_empty() {
			self.render_partially()?;
		}

		let cx = &mut Ctx::active(&mut self.core);
		if collision && !ADAPTOR.collision.get() {
			act!(mgr:peek, cx, true)?; // Reload preview if collision is resolved
		} else if preview_rect != LAYOUT.get().preview {
			act!(mgr:peek, cx)?; // Reload preview if layout changed
		}

		guard.finish(self.core.cursor());
		succ!();
	}

	pub(crate) fn render_partially(&mut self) -> Result<Data> {
		let Some(mut term) = self.core.term.take() else { succ!() };
		if !term.can_partial() {
			self.core.term = Some(term);
			self.need_render = /* normal */ 2;
			return self.render();
		}

		let guard = SyncGuard::enter();
		Self::scope(&mut self.core, term, |core, term| {
			term.draw_partial(|f| {
				f.render_widget(crate::tasks::Progress::new(core), f.area());
				f.render_widget(crate::notify::Notify::new(core), f.area());
			})
		})?;

		guard.finish(self.core.cursor());
		succ!();
	}

	fn render_forcibly(buffer: &mut Buffer) {
		let area = buffer.area;
		let image_area = ADAPTOR.shown_area();

		for y in area.top()..area.bottom() {
			for x in area.left()..area.right() {
				if image_area.is_some_and(|area| area.contains(Position { x, y })) {
					continue;
				}

				buffer[(x, y)].set_diff_option(CellDiffOption::AlwaysUpdate);
			}
		}
	}

	fn scope<F>(core: &mut Core, term: Raterm, f: F) -> Result<()>
	where
		F: FnOnce(&mut Core, &mut Raterm) -> io::Result<()>,
	{
		let mut guard = scopeguard::guard((core, term), |(core, term)| core.term = Some(term));
		let (core, term) = &mut *guard;

		Lives::scope(&mut Ctx::active(core), |cx| {
			runtime_scope!(cx, "root", { Ok(f(cx.core, term)) })
		})??;

		Ok(())
	}
}
