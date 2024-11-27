use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data};

use crate::manager::Tabs;

struct Opt {
	step: isize,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl Tabs {
	#[yazi_codegen::command]
	pub fn swap(&mut self, opt: Opt) {
		let idx = self.absolute(opt.step);
		if idx == self.cursor {
			return;
		}

		self.items.swap(self.cursor, idx);
		self.set_idx(idx);
		render!();
	}
}
