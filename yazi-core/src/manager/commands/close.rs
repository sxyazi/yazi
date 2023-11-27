use yazi_shared::event::Exec;

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Manager {
	pub fn close(&mut self, _: impl Into<Opt>, tasks: &Tasks) -> bool {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.idx);
		}
		self.quit((), tasks)
	}
}
