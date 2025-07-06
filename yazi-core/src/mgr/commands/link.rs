use yazi_parser::mgr::LinkOpt;

use crate::{mgr::Mgr, tasks::Tasks};

impl Mgr {
	#[yazi_codegen::command]
	pub fn link(&mut self, opt: LinkOpt, tasks: &Tasks) {
		if self.yanked.cut {
			return;
		}

		tasks.file_link(&self.yanked, self.cwd(), opt.relative, opt.force);
	}
}
