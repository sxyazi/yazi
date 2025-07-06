use yazi_macro::render;
use yazi_parser::confirm::CloseOpt;

use crate::confirm::Confirm;

impl Confirm {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: CloseOpt) {
		if let Some(cb) = self.callback.take() {
			_ = cb.send(opt.submit);
		}

		self.visible = false;
		render!();
	}
}
