use yazi_shared::{event::{Cmd, Data}, render};

use crate::confirm::Confirm;

pub struct Opt {
	step: isize,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl Confirm {
	fn next(&mut self, step: usize) {
		let len = self.message_num_lines();
		if len == 0 {
			return;
		}

		let old = self.vertical_scroll;
		self.vertical_scroll = (self.vertical_scroll + step).min(len - 1);

		render!(old != self.vertical_scroll);
	}

	fn prev(&mut self, step: usize) {
		let old = self.vertical_scroll;

		self.vertical_scroll -= step.min(self.vertical_scroll);

		render!(old != self.vertical_scroll);
	}

	pub fn arrow(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if opt.step > 0 { self.next(opt.step as usize) } else { self.prev(opt.step.unsigned_abs()) }
	}
}
