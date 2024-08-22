use yazi_shared::{event::{Cmd, Data}, render};

use crate::{confirm::Confirm, manager::Manager};

pub struct Opt {
	step: isize,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl Confirm {
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

	pub fn arrow(&mut self, opt: impl Into<Opt>, manager: &Manager) {
		let opt = opt.into() as Opt;
		if opt.step > 0 {
			self.next(opt.step as usize, manager.area(self.position).width)
		} else {
			self.prev(opt.step.unsigned_abs())
		}
	}
}
