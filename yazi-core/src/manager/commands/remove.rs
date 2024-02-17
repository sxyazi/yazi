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
		if !self.active_mut().try_escape_visual() {
			return;
		}

		let opt = opt.into() as Opt;
		let targets = self.selected_or_hovered().into_iter().cloned().collect();
		tasks.file_remove(targets, opt.force, opt.permanently);
	}
}
