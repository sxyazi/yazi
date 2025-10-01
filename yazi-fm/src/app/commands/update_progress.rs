use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::{act, render, succ};
use yazi_parser::app::UpdateProgressOpt;
use yazi_shared::data::Data;

use crate::app::App;

impl App {
	pub(crate) fn update_progress(&mut self, opt: UpdateProgressOpt) -> Result<Data> {
		// Update the progress of all tasks.
		let tasks = &mut self.core.tasks;
		let progressed = tasks.summary != opt.summary;
		tasks.summary = opt.summary;

		// If the task manager is visible, update the snaps with a full render.
		if tasks.visible {
			let new = tasks.paginate();
			if tasks.snaps != new {
				tasks.snaps = new;
				let cx = &mut Ctx::active(&mut self.core);
				act!(tasks:arrow, cx)?;
				succ!(render!());
			}
		}

		if !progressed {
			succ!()
		} else if tasks.summary.total == 0 {
			succ!(render!())
		} else {
			act!(render_partially, self)
		}
	}
}
