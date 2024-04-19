use yazi_shared::{event::Cmd, render};

use crate::manager::Tabs;

pub struct Opt {
	idx: usize,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self { idx: c.take_first_str().and_then(|i| i.parse().ok()).unwrap_or(0) }
	}
}

impl From<usize> for Opt {
	fn from(idx: usize) -> Self { Self { idx } }
}

impl Tabs {
	pub fn close(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		let len = self.items.len();
		if len < 2 || opt.idx >= len {
			return;
		}

		self.items.remove(opt.idx).shutdown();
		if opt.idx <= self.cursor {
			self.set_idx(self.absolute(1));
		}

		self.reorder();
		render!();
	}
}
