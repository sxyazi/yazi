use yazi_macro::render;
use yazi_parser::confirm::ArrowOpt;

use crate::{confirm::Confirm, mgr::Mgr};

impl Confirm {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: ArrowOpt, mgr: &Mgr) {
		let area = mgr.area(self.position);
		let len = self.list.line_count(area.width);

		let old = self.offset;
		self.offset = opt.step.add(self.offset, len, area.height as _);

		render!(old != self.offset);
	}
}
