use yazi_shared::{event::Cmd, render};

use crate::manager::{Manager, Yanked};

pub struct Opt {
	cut: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { cut: c.bool("cut") } }
}

impl Manager {
	pub fn yank(&mut self, opt: impl Into<Opt>) {
		if !self.active_mut().try_escape_visual() {
			return;
		}

		self.yanked = Yanked::new(opt.into().cut, self.selected_or_hovered(false).cloned().collect());
		render!(self.yanked.catchup_revision(true));

		self.active_mut().escape_select();
	}
}
