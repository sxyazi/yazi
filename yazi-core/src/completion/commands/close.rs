use yazi_config::keymap::{Exec, KeymapLayer};

use crate::{completion::Completion, emit, input::Input};

pub struct Opt {
	submit: bool,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self { Self { submit: e.named.contains_key("submit") } }
}

impl Completion {
	#[inline]
	pub fn _close() {
		emit!(Call(Exec::call("close", vec![]).vec(), KeymapLayer::Completion));
	}

	pub fn close(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;
		if opt.submit {
			Input::_complete(self.selected(), self.ticket);
		}

		self.caches.clear();
		self.visible = false;
		true
	}
}
