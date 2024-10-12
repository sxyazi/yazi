use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

struct Opt {
	force:  bool,
	follow: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { force: c.bool("force"), follow: c.bool("follow") } }
}

impl Manager {
	#[yazi_codegen::command]
	pub fn hardlink(&mut self, opt: Opt, tasks: &Tasks) {
		if self.yanked.cut {
			return;
		}

		tasks.file_hardlink(&self.yanked, self.cwd(), opt.force, opt.follow);
	}
}
