use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	relative: bool,
	force:    bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		Self { relative: c.named.contains_key("relative"), force: c.named.contains_key("force") }
	}
}

impl Manager {
	pub fn link(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		if self.yanked.cut {
			return;
		}

		let opt = opt.into() as Opt;
		tasks.file_link(&self.yanked, self.cwd(), opt.relative, opt.force);
	}
}
