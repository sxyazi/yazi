use yazi_dds::Pubsub;
use yazi_shared::{event::Cmd, render};

use crate::manager::{Manager, Yanked};

pub struct Opt {
	cut: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { cut: c.get_bool("cut") } }
}

impl Manager {
	pub fn yank(&mut self, opt: impl Into<Opt>) {
		if !self.active_mut().try_escape_visual() {
			return;
		}

		self.yanked =
			Yanked { cut: opt.into().cut, urls: self.selected_or_hovered(false).cloned().collect() };

		self.active_mut().escape_select();
		Pubsub::pub_from_yank(self.yanked.cut, &self.yanked.urls);

		render!();
	}
}
