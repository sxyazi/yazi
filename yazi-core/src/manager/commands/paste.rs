use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	force:  bool,
	follow: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		Self { force: c.named.contains_key("force"), follow: c.named.contains_key("follow") }
	}
}

impl Manager {
	pub fn paste(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		let dest = self.cwd();
		let (cut, ref src) = self.yanked;

		let opt = opt.into() as Opt;
		if cut {
			tasks.file_cut(src, dest, opt.force);
		} else {
			tasks.file_copy(src, dest, opt.force, opt.follow);
		}
	}
}
