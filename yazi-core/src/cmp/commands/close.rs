use std::mem;

use yazi_macro::render;
use yazi_parser::cmp::CloseOpt;
use yazi_proxy::InputProxy;

use crate::cmp::Cmp;

impl Cmp {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: CloseOpt) {
		if let Some(s) = self.selected().filter(|_| opt.submit) {
			InputProxy::complete(s, self.ticket);
		}

		self.caches.clear();
		render!(mem::replace(&mut self.visible, false));
	}
}
