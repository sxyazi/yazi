use yazi_shared::event::Exec;

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	force:       bool,
	permanently: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self {
			force:       e.named.contains_key("force"),
			permanently: e.named.contains_key("permanently"),
		}
	}
}

impl Manager {
	pub fn remove(&mut self, opt: impl Into<Opt>, tasks: &Tasks) -> bool {
		let opt = opt.into() as Opt;
		let targets = self.selected().into_iter().map(|f| f.url()).collect();
		tasks.file_remove(targets, opt.force, opt.permanently)
	}
}
