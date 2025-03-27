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
		let area = mgr.area(self.position);
		let len = self.list.line_count(area.width);

		let old = self.offset;
		self.offset = opt.step.add(self.offset, len, area.height as _);

		render!(old != self.offset);
	}
}
