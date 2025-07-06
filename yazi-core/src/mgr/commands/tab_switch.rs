use yazi_macro::render;
use yazi_parser::mgr::TabSwitchOpt;

use crate::mgr::Tabs;

impl Tabs {
	#[yazi_codegen::command]
	pub fn switch(&mut self, opt: TabSwitchOpt) {
		let idx = if opt.relative {
			opt.step.saturating_add_unsigned(self.cursor).rem_euclid(self.items.len() as _) as _
		} else {
			opt.step as usize
		};

		if idx == self.cursor || idx >= self.items.len() {
			return;
		}

		self.set_idx(idx);
		render!();
	}
}
