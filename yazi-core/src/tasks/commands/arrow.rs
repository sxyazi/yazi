use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::tasks::Tasks;

struct Opt {
	step: isize,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl From<isize> for Opt {
	fn from(step: isize) -> Self { Self { step } }
}

impl Tasks {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
		let old = self.cursor;
		if opt.step > 0 {
			self.cursor += 1;
		} else {
			self.cursor = self.cursor.saturating_sub(1);
		}

		let max = Self::limit().min(self.summaries.len());
		self.cursor = self.cursor.min(max.saturating_sub(1));
		render!(self.cursor != old);
	}
}
