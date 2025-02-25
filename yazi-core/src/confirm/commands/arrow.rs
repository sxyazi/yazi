use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::{confirm::Confirm, mgr::Mgr};

struct Opt {
	step: isize,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl Confirm {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt, mgr: &Mgr) {
		if opt.step > 0 {
			self.next(opt.step as usize, mgr.area(self.position).width)
		} else {
			self.prev(opt.step.unsigned_abs())
		}
	}

	fn next(&mut self, step: usize, width: u16) {
		let height = self.list.line_count(width);
		if height == 0 {
			return;
		}

		let old = self.offset;
		self.offset = (self.offset + step).min(height - 1);

		render!(old != self.offset);
	}

	fn prev(&mut self, step: usize) {
		let old = self.offset;
		self.offset -= step.min(self.offset);

		render!(old != self.offset);
	}
}
