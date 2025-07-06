use yazi_macro::render;
use yazi_parser::tasks::ArrowOpt;

use crate::tasks::Tasks;

impl Tasks {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: ArrowOpt) {
		let old = self.cursor;
		self.cursor = opt.step.add(self.cursor, self.summaries.len(), Self::limit());

		render!(self.cursor != old);
	}
}
