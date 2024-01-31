use yazi_shared::{event::Cmd, render};

use crate::manager::Manager;

pub struct Opt {
	cut: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { cut: c.named.contains_key("cut") } }
}

impl Manager {
	pub fn yank(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		self.yanked.0 = opt.cut;
		self.yanked.1 = self.selected().into_iter().map(|f| f.url()).collect();
		render!();
	}
}
