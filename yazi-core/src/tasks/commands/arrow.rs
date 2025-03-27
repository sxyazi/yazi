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
	fn from(step: isize) -> Self { Self { step: step.into() } }
}

impl Tasks {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
		let max = Self::limit().min(self.summaries.len());
		let old = self.cursor;
		let new = opt.step.add(self.cursor, max, max);
		if new > old {
			self.cursor += 1;
		} else {
			self.cursor = self.cursor.saturating_sub(1);
		}

		self.cursor = self.cursor.min(max.saturating_sub(1));
		render!(self.cursor != old);
	}
}
