use yazi_shared::event::Exec;

use crate::manager::Manager;

pub struct Opt {
	cut: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { cut: e.named.contains_key("cut") } }
}

impl Manager {
	pub fn yank(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;

		self.yanked.0 = opt.cut;
		self.yanked.1 = self.selected().into_iter().map(|f| f.url()).collect();
		true
	}
}
