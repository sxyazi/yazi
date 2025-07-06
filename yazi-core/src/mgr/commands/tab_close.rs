use yazi_macro::render;
use yazi_parser::mgr::TabCloseOpt;

use crate::mgr::Tabs;

impl Tabs {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: TabCloseOpt) {
		let len = self.items.len();
		if len < 2 || opt.idx >= len {
			return;
		}

		self.items.remove(opt.idx).shutdown();
		if opt.idx > self.cursor {
			self.set_idx(self.cursor);
		} else {
			self.set_idx(usize::min(self.cursor + 1, self.items.len() - 1));
		}

		render!();
	}
}
