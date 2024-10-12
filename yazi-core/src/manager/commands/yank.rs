use yazi_macro::render;
use yazi_shared::event::Cmd;

use crate::manager::{Manager, Yanked};

struct Opt {
	cut: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { cut: c.bool("cut") } }
}

impl Manager {
	#[yazi_codegen::command]
	pub fn yank(&mut self, opt: Opt) {
		if !self.active_mut().try_escape_visual() {
			return;
		}

		self.yanked = Yanked::new(opt.cut, self.selected_or_hovered(false).cloned().collect());
		render!(self.yanked.catchup_revision(true));

		self.active_mut().escape_select();
	}
}
