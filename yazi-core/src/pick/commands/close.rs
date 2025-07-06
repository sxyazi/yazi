use anyhow::anyhow;
use yazi_macro::render;
use yazi_parser::pick::CloseOpt;

use crate::pick::Pick;

impl Pick {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: CloseOpt) {
		if let Some(cb) = self.callback.take() {
			_ = cb.send(if opt.submit { Ok(self.cursor) } else { Err(anyhow!("canceled")) });
		}

		self.cursor = 0;
		self.offset = 0;
		self.visible = false;
		render!();
	}
}
