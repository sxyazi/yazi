use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	force:  bool,
	follow: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { force: c.bool("force"), follow: c.bool("follow") } }
}

impl Manager {
	pub fn hardlink(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		if self.yanked.cut {
			return;
		}

		let opt = opt.into() as Opt;
		tasks.file_hardlink(&self.yanked, self.cwd(), opt.force, opt.follow);
	}
}
