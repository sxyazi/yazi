use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{completion::Completion, emit};

pub struct Opt(bool);

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self(e.named.contains_key("submit")) }
}

impl From<bool> for Opt {
	fn from(b: bool) -> Self { Self(b) }
}

impl Completion {
	pub fn close(&mut self, opt: impl Into<Opt>) -> bool {
		let submit = opt.into().0;
		if submit {
			emit!(Call(
				Exec::call("complete", vec![self.items[self.cursor].to_owned()])
					.with_bool("apply", true)
					.with("ticket", self.ticket)
					.vec(),
				KeymapLayer::Input
			));
		}

		self.cursor = 0;
		self.visible = false;
		true
	}
}
