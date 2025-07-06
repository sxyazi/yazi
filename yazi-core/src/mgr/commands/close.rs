use yazi_parser::mgr::CloseOpt;

use crate::{mgr::Mgr, tasks::Tasks};

impl Mgr {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: CloseOpt, tasks: &Tasks) {
		if self.tabs.len() > 1 {
			return self.tabs.close(self.tabs.cursor);
		}
		self.quit(opt.0, tasks);
	}
}
