use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

impl Manager {
	pub fn close(&mut self, _: Cmd, tasks: &Tasks) {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.cursor);
		}
		self.quit((), tasks);
	}
}
