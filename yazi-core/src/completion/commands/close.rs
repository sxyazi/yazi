use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{completion::Completion, emit};

pub struct Opt {
	submit: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { submit: e.named.contains_key("submit") } }
}

impl Completion {
	pub fn close(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;
		if opt.submit {
			emit!(Call(
				Exec::call("complete", vec![self.selected().into()]).with("ticket", self.ticket).vec(),
				KeymapLayer::Input
			));
		}

		self.caches.clear();
		self.visible = false;
		true
	}
}
