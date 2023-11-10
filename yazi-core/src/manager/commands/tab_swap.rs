use yazi_config::keymap::Exec;

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
	pub fn swap(&mut self, opt: impl Into<Opt>) -> bool {
		let idx = self.absolute(opt.into().step);
		if idx == self.idx {
			return false;
		}

		self.items.swap(self.idx, idx);
		self.set_idx(idx);
		true
	}
}
