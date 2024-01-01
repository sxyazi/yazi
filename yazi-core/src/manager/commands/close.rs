use yazi_shared::{event::Exec, render};

use crate::{manager::Manager, tasks::Tasks};

impl Manager {
	pub fn close(&mut self, _: &Exec, tasks: &Tasks) {
		if self.tabs.len() > 1 {
			return render!(self.tabs.close(self.tabs.idx));
		}
		self.quit((), tasks);
	}
}
