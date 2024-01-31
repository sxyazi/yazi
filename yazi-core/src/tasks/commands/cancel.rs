use yazi_shared::{event::Cmd, render};

use crate::tasks::Tasks;

impl Tasks {
	pub fn cancel(&mut self, _: Cmd) {
		let id = self.scheduler.running.lock().get_id(self.cursor);
		if id.map(|id| self.scheduler.cancel(id)) != Some(true) {
			return;
		}

		let len = self.scheduler.running.lock().len();
		self.cursor = self.cursor.min(len.saturating_sub(1));
		render!();
	}
}
