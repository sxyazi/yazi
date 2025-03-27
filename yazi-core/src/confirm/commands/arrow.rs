use yazi_fs::Step;
use yazi_macro::render;
use yazi_shared::event::CmdCow;

use crate::{confirm::Confirm, mgr::Mgr};

struct Opt {
	step: Step,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { step: c.first().and_then(|d| d.try_into().ok()).unwrap_or_default() }
	}
}

impl Confirm {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt, mgr: &Mgr) {
		let width = mgr.area(self.position).width;
		let height = self.list.line_count(width);
		if height == 0 {
			return;
		}

		let old = self.offset;
		let new = opt.step.add(self.offset, height, height);
		self.offset = new.min(height - 1);

		render!(old != self.offset);
	}
}
