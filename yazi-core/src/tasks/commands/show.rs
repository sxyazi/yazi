use yazi_macro::render;
use yazi_parser::tasks::ShowOpt;

use crate::tasks::Tasks;

impl Tasks {
	#[yazi_codegen::command]
	pub fn show(&mut self, _: ShowOpt) {
		if self.visible {
			return;
		}

		self.visible = true;
		self.summaries = self.paginate();

		self.arrow(0);
		render!();
	}
}
