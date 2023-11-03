use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{completion::Completion, emit};

pub struct Opt(bool);

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self(e.named.contains_key("submit")) }
}

impl Completion {
	pub fn close(&mut self, opt: impl Into<Opt>) -> bool {
		let submit = opt.into().0;
		if submit {
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
