use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::help::Help;

struct Opt {
	step: isize,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}
impl From<isize> for Opt {
	fn from(step: isize) -> Self { Self { step } }
}

impl Help {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, opt: Opt) {
		let max = self.bindings.len().saturating_sub(1);
		self.offset = self.offset.min(max);
		self.cursor = self.cursor.min(max);

		if opt.step > 0 {
			self.next(opt.step as usize);
		} else {
			self.prev(opt.step.unsigned_abs());
		}
	}

	fn next(&mut self, new: usize) {
		let old = self.cursor;
		self.cursor = new;

		let (len, limit) = (self.bindings.len(), Self::limit());
		self.offset = if self.cursor < (self.offset + limit).min(len).saturating_sub(5) {
			self.offset.min(len.saturating_sub(1))
		} else {
			len.saturating_sub(limit).min(self.offset + self.cursor - old)
		};

		render!(old != self.cursor);
	}

	fn prev(&mut self, new: usize) {
		let old = self.cursor;
		self.cursor = new;

		self.offset = if self.cursor < self.offset + 5 {
			self.offset.saturating_sub(old - self.cursor)
		} else {
			self.offset.min(self.bindings.len().saturating_sub(1))
		};

		render!(old != self.cursor);
	}
}
