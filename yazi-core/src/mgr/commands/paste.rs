use yazi_parser::mgr::PasteOpt;

use crate::{mgr::Mgr, tasks::Tasks};

impl Mgr {
	#[yazi_codegen::command]
	pub fn paste(&mut self, opt: PasteOpt, tasks: &Tasks) {
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
