use yazi_shared::event::CmdCow;

use crate::{mgr::Mgr, tasks::Tasks};

struct Opt {
	force:  bool,
	follow: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { force: c.bool("force"), follow: c.bool("follow") } }
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn hardlink(&mut self, opt: Opt, tasks: &Tasks) {
		if self.yanked.cut {
			return;
		}

		tasks.file_hardlink(&self.yanked, self.cwd(), opt.force, opt.follow);
	}
}
