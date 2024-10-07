use yazi_proxy::InputProxy;
use yazi_shared::{event::Cmd, render};

use crate::completion::Completion;

pub struct Opt {
	submit: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { submit: c.bool("submit") } }
}
impl From<bool> for Opt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl Completion {
	#[yazi_macro::command]
	pub fn close(&mut self, opt: Opt) {
		if let Some(s) = self.selected().filter(|_| opt.submit) {
			InputProxy::complete(s, self.ticket);
		}

		self.caches.clear();
		self.visible = false;
		render!();
	}
}
