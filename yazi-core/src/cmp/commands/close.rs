use std::mem;

use yazi_macro::render;
use yazi_proxy::InputProxy;
use yazi_shared::event::CmdCow;

use crate::cmp::Cmp;

struct Opt {
	submit: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { submit: c.bool("submit") } }
}
impl From<bool> for Opt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl Cmp {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt) {
		if let Some(s) = self.selected().filter(|_| opt.submit) {
			InputProxy::complete(s, self.ticket);
		}

		self.caches.clear();
		render!(mem::replace(&mut self.visible, false));
	}
}
