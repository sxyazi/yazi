use anyhow::Result;
use yazi_actor::Ctx;
use yazi_macro::{act, render, succ};
use yazi_parser::app::UpdateProgressOpt;
use yazi_shared::event::Data;

use crate::app::App;

impl App {
	pub(crate) fn update_progress(&mut self, opt: UpdateProgressOpt) -> Result<Data> {
		// Update the progress of all tasks.
		let tasks = &mut self.core.tasks;
		let progressed = tasks.progress != opt.progress;
		tasks.progress = opt.progress;

		// If the task manager is visible, update the summaries with a complete render.
		if tasks.visible {
			let new = tasks.paginate();
			if tasks.summaries != new {
				tasks.summaries = new;
				act!(tasks:arrow, &mut Ctx::active(&mut self.core))?;
				succ!(render!());
			}
		}

		if !progressed {
			succ!()
		} else if tasks.progress.total == 0 {
			succ!(render!())
		} else {
			act!(render_partially, self)
		}
	}
}
