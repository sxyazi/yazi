use yazi_shared::event::Exec;

use crate::{manager::Manager, tasks::Tasks};

impl Manager {
	pub fn close(&mut self, _: &Exec, tasks: &Tasks) -> bool {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.idx);
		}
		self.quit((), tasks)
	}
}
