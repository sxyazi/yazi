use ratatui::backend::Backend;
use yazi_core::tasks::TasksProgress;
use yazi_shared::{event::Cmd, render};

use crate::{app::App, components::Progress, lives::Lives};

pub struct Opt {
	progress: TasksProgress,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self { progress: c.take_any("progress").ok_or(())? })
	}
}

impl App {
	pub(crate) fn update_progress(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		// Update the progress of all tasks.
		let tasks = &mut self.cx.tasks;
		tasks.progress = opt.progress;

		// If the task manager is visible, update the summaries with a complete render.
		if tasks.visible {
			let new = tasks.paginate();
			if new.len() != tasks.summaries.len()
				|| new.iter().zip(&tasks.summaries).any(|(a, b)| a.name != b.name)
			{
				tasks.summaries = new;
				tasks.arrow(0);
				return render!();
			}
		}

		// Otherwise, only partially update the progress.
		let Some(term) = &mut self.term else {
			return;
		};

		_ = Lives::scope(&self.cx, |_| {
			for patch in Progress::partial_render(term.current_buffer_mut()) {
				term.backend_mut().draw(patch.iter().map(|(x, y, cell)| (*x, *y, cell)))?;
				if let Some(pos) = self.cx.cursor() {
					term.show_cursor()?;
					term.set_cursor_position(pos)?;
				}
				term.backend_mut().flush()?;
			}
			Ok(())
		});
	}
}
