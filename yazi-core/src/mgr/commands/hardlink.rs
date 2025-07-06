use yazi_parser::mgr::HardlinkOpt;

use crate::{mgr::Mgr, tasks::Tasks};

impl Mgr {
	#[yazi_codegen::command]
	pub fn hardlink(&mut self, opt: HardlinkOpt, tasks: &Tasks) {
		if self.yanked.cut {
			return;
		}

		tasks.file_hardlink(&self.yanked, self.cwd(), opt.force, opt.follow);
	}
}
