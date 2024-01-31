use yazi_shared::{emit, event::Cmd, render, Layer};

use crate::{completion::Completion, input::Input};

pub struct Opt {
	submit: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { submit: c.named.contains_key("submit") } }
}

impl Completion {
	#[inline]
	pub fn _close() {
		emit!(Call(Cmd::new("close"), Layer::Completion));
	}

	pub fn close(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		if let Some(s) = self.selected().filter(|_| opt.submit) {
			Input::_complete(s, self.ticket);
		}

		self.caches.clear();
		self.visible = false;
		render!();
	}
}
