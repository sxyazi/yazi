use yazi_shared::{event::Exec, render};

use crate::manager::Tabs;

pub struct Opt {
	step:     isize,
	relative: bool,
}

impl From<Exec> for Opt {
	fn from(mut e: Exec) -> Self {
		Self {
			step:     e.take_first().and_then(|s| s.parse().ok()).unwrap_or(0),
			relative: e.named.contains_key("relative"),
		}
	}
}

impl Tabs {
	pub fn switch(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		let idx = if opt.relative {
			(self.idx as isize + opt.step).rem_euclid(self.items.len() as isize) as usize
		} else {
			opt.step as usize
		};

		if idx == self.idx || idx >= self.items.len() {
			return;
		}

		self.set_idx(idx);
		render!();
	}
}
