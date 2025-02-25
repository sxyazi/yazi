use yazi_shared::event::CmdCow;

use crate::{mgr::Mgr, tasks::Tasks};

struct Opt {
	relative: bool,
	force:    bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { relative: c.bool("relative"), force: c.bool("force") } }
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn link(&mut self, opt: Opt, tasks: &Tasks) {
		if self.yanked.cut {
			return;
		}

		tasks.file_link(&self.yanked, self.cwd(), opt.relative, opt.force);
	}
}
