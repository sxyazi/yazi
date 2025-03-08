use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::mgr::Tabs;

struct Opt {
	idx: usize,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { idx: c.first().and_then(Data::as_usize).unwrap_or(0) } }
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
		if opt.idx > self.cursor {
			self.set_idx(self.cursor);
		} else {
			self.set_idx(usize::min(self.cursor + 1, self.items.len() - 1));
		}

		render!();
	}
}
