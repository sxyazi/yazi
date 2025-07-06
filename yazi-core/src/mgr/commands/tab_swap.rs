use yazi_macro::render;
use yazi_parser::mgr::TabSwapOpt;

use crate::mgr::Tabs;

impl Tabs {
	#[yazi_codegen::command]
	pub fn swap(&mut self, opt: TabSwapOpt) {
		let idx = opt.step.saturating_add_unsigned(self.cursor).rem_euclid(self.items.len() as _) as _;
		if idx == self.cursor {
			return;
		}

		self.items.swap(self.cursor, idx);
		self.set_idx(idx);
		render!();
	}
}
