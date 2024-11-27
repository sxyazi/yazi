use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::confirm::Confirm;

struct Opt {
	submit: bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { submit: c.bool("submit") } }
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
