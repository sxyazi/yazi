use yazi_shared::event::Cmd;

use crate::{manager::Manager, tasks::Tasks};

struct Opt {
	relative: bool,
	force:    bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { relative: c.bool("relative"), force: c.bool("force") } }
}

impl Manager {
	#[yazi_codegen::command]
	pub fn link(&mut self, opt: Opt, tasks: &Tasks) {
		if self.yanked.cut {
			return;
		}

		tasks.file_link(&self.yanked, self.cwd(), opt.relative, opt.force);
	}
}
