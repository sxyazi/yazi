use yazi_shared::{event::Cmd, render};

use crate::manager::{Manager, Yanked};

pub struct Opt {
	cut: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { cut: c.named.contains_key("cut") } }
}

impl Manager {
	pub fn yank(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		self.yanked =
			Yanked { cut: opt.cut, urls: self.selected_or_hovered().into_iter().cloned().collect() };
		render!();
	}
}
