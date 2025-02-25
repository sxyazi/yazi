use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::cmp::Cmp;

struct Opt {
	step: isize,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl Cmp {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
		if opt.step > 0 {
			self.next(opt.step as usize);
		} else {
			self.prev(opt.step.unsigned_abs());
		}
	}

	fn next(&mut self, step: usize) {
		let len = self.cands.len();
		if len == 0 {
			return;
		}

		let old = self.cursor;
		self.cursor = (self.cursor + step).min(len - 1);

		let limit = self.limit();
		if self.cursor >= len.min(self.offset + limit) {
			self.offset = len.saturating_sub(limit).min(self.offset + self.cursor - old);
		}

		render!(old != self.cursor);
	}

	fn prev(&mut self, step: usize) {
		let old = self.cursor;
		self.cursor = self.cursor.saturating_sub(step);

		if self.cursor < self.offset {
			self.offset = self.offset.saturating_sub(old - self.cursor);
		}

		render!(old != self.cursor);
	}
}
