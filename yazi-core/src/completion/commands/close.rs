use yazi_proxy::InputProxy;
use yazi_shared::{event::Cmd, render};

use crate::completion::Completion;

pub struct Opt {
	submit: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { submit: c.named.contains_key("submit") } }
}

impl Completion {
	pub fn close(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		if let Some(s) = self.selected().filter(|_| opt.submit) {
			InputProxy::complete(s, self.ticket);
		}

		self.caches.clear();
		self.visible = false;
		render!();
	}
}
