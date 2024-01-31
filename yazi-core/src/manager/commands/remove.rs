use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	force:       bool,
	permanently: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		Self {
			force:       c.named.contains_key("force"),
			permanently: c.named.contains_key("permanently"),
		}
	}
}

impl Manager {
	pub fn remove(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		let opt = opt.into() as Opt;
		let targets = self.selected().into_iter().map(|f| f.url()).collect();
		tasks.file_remove(targets, opt.force, opt.permanently);
	}
}
