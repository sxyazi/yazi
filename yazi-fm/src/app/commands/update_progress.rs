use yazi_core::tasks::TasksProgress;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::app::App;

pub struct Opt {
	progress: TasksProgress,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { progress: c.take_any("progress").ok_or(())? })
	}
}

impl App {
	pub(crate) fn update_progress(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else { return };

		// Update the progress of all tasks.
		let tasks = &mut self.cx.tasks;
		let progressed = tasks.progress != opt.progress;
		tasks.progress = opt.progress;

		// If the task manager is visible, update the summaries with a complete render.
		if tasks.visible {
			let new = tasks.paginate();
			if tasks.summaries != new {
				tasks.summaries = new;
				tasks.arrow(0);
				return render!();
			}
		}

		if !progressed {
		} else if tasks.progress.total == 0 {
			render!();
		} else {
			self.render_partially();
		}
	}
}
