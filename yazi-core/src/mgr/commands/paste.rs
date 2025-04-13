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
	pub fn paste(&mut self, opt: Opt, tasks: &Tasks) {
		let (src, dest) = (self.yanked.iter().collect::<Vec<_>>(), self.cwd());

		if self.yanked.cut {
			tasks.file_cut(&src, dest, opt.force);

			self.tabs.iter_mut().for_each(|t| _ = t.selected.remove_many(&src));
			self.unyank(());
		} else {
			tasks.file_copy(&src, dest, opt.force, opt.follow);
		}
	}
}
