use yazi_shared::{event::Exec, render};

use crate::manager::Tabs;

pub struct Opt {
	step: isize,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self { step: e.args.first().and_then(|s| s.parse().ok()).unwrap_or(0) }
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
