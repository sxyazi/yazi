use ratatui::backend::Backend;
use yazi_core::tasks::TasksProgress;
use yazi_shared::event::Cmd;

use crate::{app::App, components::Progress, lives::Lives};

pub struct Opt {
	progress: TasksProgress,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self { progress: c.take_data().ok_or(())? })
	}
}

impl App {
	pub(crate) fn update_progress(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		self.cx.tasks.progress = opt.progress;
		let Some(term) = &mut self.term else {
			return;
		};

		Lives::partial_scope(&self.cx, |_| {
			for patch in Progress::partial_render(term.current_buffer_mut()) {
				term.backend_mut().draw(patch.iter().map(|(x, y, cell)| (*x, *y, cell))).ok();
				if let Some((x, y)) = self.cx.cursor() {
					term.show_cursor().ok();
					term.set_cursor(x, y).ok();
				}
				term.backend_mut().flush().ok();
			}
		});
	}
}
