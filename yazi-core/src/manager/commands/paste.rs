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
	#[yazi_macro::command]
	pub fn paste(&mut self, opt: Opt, tasks: &Tasks) {
		let (src, dest) = (self.yanked.iter().collect::<Vec<_>>(), self.cwd());

		if self.yanked.cut {
			tasks.file_cut(&src, dest, opt.force);

			self.tabs.iter_mut().for_each(|t| _ = t.selected.remove_many(&src, false));
			self.unyank(());
		} else {
			tasks.file_copy(&src, dest, opt.force, opt.follow);
		}
	}
}
