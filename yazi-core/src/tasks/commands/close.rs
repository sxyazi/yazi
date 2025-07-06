use yazi_macro::render;
use yazi_parser::tasks::CloseOpt;

use crate::tasks::Tasks;

impl Tasks {
	#[yazi_codegen::command]
	pub fn close(&mut self, _: CloseOpt) {
		if !self.visible {
			return;
		}

		self.visible = false;
		self.summaries = Vec::new();

		self.arrow(0);
		render!();
	}
}
