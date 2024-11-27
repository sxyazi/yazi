use yazi_shared::event::CmdCow;

use crate::{manager::Manager, tasks::Tasks};

impl Manager {
	pub fn close(&mut self, _: CmdCow, tasks: &Tasks) {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.cursor);
		}
		self.quit((), tasks);
	}
}
