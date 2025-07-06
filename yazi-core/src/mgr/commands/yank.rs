use yazi_macro::render;
use yazi_parser::mgr::YankOpt;

use crate::mgr::{Mgr, Yanked};

impl Mgr {
	#[yazi_codegen::command]
	pub fn yank(&mut self, opt: YankOpt) {
		if !self.active_mut().try_escape_visual() {
			return;
		}

		self.yanked = Yanked::new(opt.cut, self.selected_or_hovered().cloned().collect());
		render!(self.yanked.catchup_revision(true));

		self.active_mut().escape_select();
	}
}
