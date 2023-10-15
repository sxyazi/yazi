use crate::{manager::Manager, tasks::Tasks};

impl Manager {
	pub fn close(&mut self, tasks: &Tasks) -> bool {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.idx);
		}
		self.quit(tasks, false)
	}
}
