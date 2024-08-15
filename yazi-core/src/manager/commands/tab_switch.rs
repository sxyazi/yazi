use yazi_shared::{event::{Cmd, Data}, render};

use crate::manager::Tabs;

pub struct Opt {
	step:     isize,
	relative: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self {
		Self { step: c.first().and_then(Data::as_isize).unwrap_or(0), relative: c.bool("relative") }
	}
}

impl Tabs {
	pub fn switch(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		let idx = if opt.relative {
			(self.cursor as isize + opt.step).rem_euclid(self.items.len() as isize) as usize
		} else {
			opt.step as usize
		};

		if idx == self.cursor || idx >= self.items.len() {
			return;
		}

		self.set_idx(idx);
		render!();
	}
}
