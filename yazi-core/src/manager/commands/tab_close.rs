use yazi_shared::{event::{Cmd, Data}, render};

use crate::manager::Tabs;

pub struct Opt {
	idx: usize,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { idx: c.first().and_then(Data::as_usize).unwrap_or(0) } }
}

impl From<usize> for Opt {
	fn from(idx: usize) -> Self { Self { idx } }
}

impl Tabs {
	#[yazi_codegen::command]
	pub fn close(&mut self, opt: Opt) {
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
