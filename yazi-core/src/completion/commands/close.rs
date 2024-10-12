use yazi_macro::render;
use yazi_proxy::InputProxy;
use yazi_shared::event::Cmd;

use crate::completion::Completion;

struct Opt {
	submit: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { submit: c.bool("submit") } }
}
impl From<bool> for Opt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl Completion {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt) {
		if let Some(s) = self.selected().filter(|_| opt.submit) {
			InputProxy::complete(s, self.ticket);
		}

		self.caches.clear();
		self.visible = false;
		render!();
	}
}
