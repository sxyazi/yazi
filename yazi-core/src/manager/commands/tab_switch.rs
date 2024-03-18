use yazi_shared::{event::Cmd, render};

use crate::manager::Tabs;

pub struct Opt {
	step:     isize,
	relative: bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			step:     c.take_first().and_then(|s| s.parse().ok()).unwrap_or(0),
			relative: c.named.contains_key("relative"),
		}
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
