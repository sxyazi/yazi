use yazi_shared::{event::Cmd, render};

use crate::manager::Tabs;

pub struct Opt {
	step: isize,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self { step: c.take_first().and_then(|s| s.parse().ok()).unwrap_or(0) }
	}
}

impl Tabs {
	pub fn swap(&mut self, opt: impl Into<Opt>) {
		let idx = self.absolute(opt.into().step);
		if idx == self.idx {
			return;
		}

		self.items.swap(self.idx, idx);
		self.set_idx(idx);
		render!();
	}
}
