use yazi_config::keymap::Exec;

use crate::tasks::Tasks;

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Tasks {
	pub fn cancel(&mut self, _: impl Into<Opt>) -> bool {
		let id = self.scheduler.running.read().get_id(self.cursor);
		if id.map(|id| self.scheduler.cancel(id)) != Some(true) {
			return false;
		}

		let len = self.scheduler.running.read().len();
		self.cursor = self.cursor.min(len.saturating_sub(1));
		true
	}
}
