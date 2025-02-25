use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::mgr::{Mgr, Yanked};

struct Opt {
	cut: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { cut: c.bool("cut") } }
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn yank(&mut self, opt: Opt) {
		if !self.active_mut().try_escape_visual() {
			return;
		}

		self.yanked = Yanked::new(opt.cut, self.selected_or_hovered().cloned().collect());
		render!(self.yanked.catchup_revision(true));

		self.active_mut().escape_select();
	}
}
