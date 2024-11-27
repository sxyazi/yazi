use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::tasks::Tasks;

impl Tasks {
	pub fn cancel(&mut self, _: CmdCow) {
		let id = self.ongoing().lock().get_id(self.cursor);
		if id.map(|id| self.scheduler.cancel(id)) != Some(true) {
			return;
		}

		self.summaries = self.paginate();
		self.arrow(0);
		render!();
	}
}
