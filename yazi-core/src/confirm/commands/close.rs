use yazi_macro::render;
use yazi_shared::event::Cmd;

use crate::confirm::Confirm;

struct Opt {
	submit: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { submit: c.bool("submit") } }
}
impl From<bool> for Opt {
	fn from(submit: bool) -> Self { Self { submit } }
}

impl Confirm {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt) {
		if let Some(cb) = self.callback.take() {
			_ = cb.send(opt.submit);
		}

		self.visible = false;
		render!();
	}
}
