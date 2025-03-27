use yazi_fs::Step;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::tasks::Tasks;

struct Opt {
	step: Step,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(|d| d.try_into().ok()).unwrap_or_default() }
	}
}

impl From<isize> for Opt {
	fn from(n: isize) -> Self { Self { step: n.into() } }
}

impl Tasks {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
		let old = self.cursor;
		self.cursor = opt.step.add(self.cursor, self.summaries.len(), Self::limit());

		render!(self.cursor != old);
	}
}
